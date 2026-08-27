use actix_web::*;
use actix_ws::Closed;
use autometrics::autometrics;
use serde::Deserialize;
use tracing::{Level, debug, warn};

use service::app_data::{AppData, QueryError};
use service::contest_state::ContestState;

#[derive(Deserialize, Debug)]
struct ContestQuery {
    pub contest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SecretQuery {
    secret: String,
}

const API_KEY: &str = "apikey";

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service((
        get_contest,
        get_timer,
        get_config,
        get_allruns_ws,
        get_allruns_secret,
        update_contest,
    ));
}

/// Serialize a value and send it over a websocket.
/// Returns false when the connection closed and the caller should stop.
pub(crate) async fn send_json<T: serde::Serialize>(
    session: &mut actix_ws::Session,
    value: &T,
) -> bool {
    match serde_json::to_string(value) {
        Ok(text) => match session.text(text).await {
            Ok(()) => true,
            Err(Closed) => false,
        },
        Err(err) => {
            warn!(?err, "failed serializing");
            true
        }
    }
}

fn map_query<T: serde::Serialize>(result: Result<T, QueryError>) -> HttpResponse {
    match result {
        Ok(value) => HttpResponse::Ok().json(value),
        Err(QueryError::Forbidden) => HttpResponse::Forbidden().finish(),
        Err(QueryError::NotFound) => HttpResponse::NotFound().finish(),
    }
}

#[get("/contest")]
async fn get_contest(
    data: web::Data<AppData>,
    contest: web::Query<ContestQuery>,
) -> HttpResponse {
    get_contest_fn(
        data.get_ref(),
        contest.into_inner().contest.unwrap_or_default().as_str(),
    )
    .await
}

#[tracing::instrument(level = Level::DEBUG, skip(data), ret)]
#[autometrics]
async fn get_contest_fn(data: &AppData, sede_config: &str) -> HttpResponse {
    map_query(data.contest_file(sede_config).await)
}

#[get("/config")]
async fn get_config(data: web::Data<AppData>, contest: web::Query<ContestQuery>) -> HttpResponse {
    get_config_fn(
        data.get_ref(),
        contest.into_inner().contest.unwrap_or_default().as_str(),
    )
    .await
}

#[tracing::instrument(level = Level::DEBUG, skip(data), ret)]
#[autometrics]
async fn get_config_fn(data: &AppData, sede_config: &str) -> HttpResponse {
    map_query(data.config_contest(sede_config).await)
}

#[tracing::instrument(level = Level::DEBUG, skip(data), ret)]
#[autometrics]
async fn get_allruns_secret_fn(data: &AppData, sede_config: &str, secret: &str) -> HttpResponse {
    match data.secret_runs(sede_config, secret).await {
        Ok(runs) => HttpResponse::Ok().json(runs),
        Err(_) => HttpResponse::Forbidden().finish(),
    }
}

#[get("/allruns_secret")]
async fn get_allruns_secret(
    data: web::Data<AppData>,
    query: web::Query<SecretQuery>,
    contest: web::Query<ContestQuery>,
) -> HttpResponse {
    get_allruns_secret_fn(
        data.get_ref(),
        contest.into_inner().contest.unwrap_or_default().as_str(),
        &query.secret,
    )
    .await
}

#[autometrics]
#[tracing::instrument(level = Level::DEBUG, skip(data, body), ret)]
#[get("/allruns_ws")]
async fn get_allruns_ws(
    data: web::Data<AppData>,
    req: HttpRequest,
    body: web::Payload,
    contest: web::Query<ContestQuery>,
) -> Result<HttpResponse, Error> {
    let (response, mut session, _msg_stream) = actix_ws::handle(&req, body)?;
    let mut runs_rx = data.runs_subscribe();

    let sede = data.sede_title(contest.into_inner().contest.unwrap_or_default().as_str());

    match sede {
        None => Ok(HttpResponse::Forbidden().finish()),
        Some(sede) => {
            actix_web::rt::spawn(async move {
                loop {
                    match runs_rx.recv().await {
                        Ok(r) => {
                            if sede.team_belongs_str(&r.team_login)
                                && !send_json(&mut session, &r).await
                            {
                                debug!("ws connection closed");
                                break;
                            }
                        }
                        Err(err) => {
                            warn!(?err, "recv failed");
                            break;
                        }
                    }
                }
            });
            Ok(response)
        }
    }
}

#[autometrics]
#[tracing::instrument(level = Level::DEBUG, skip(data, body), ret)]
#[get("/timer")]
async fn get_timer(
    data: web::Data<AppData>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, Error> {
    let (response, mut session, _msg_stream) = actix_ws::handle(&req, body)?;
    let mut time_rx = data.time_subscribe();

    actix_web::rt::spawn(async move {
        let mut previous = None;
        loop {
            match time_rx.recv().await {
                Ok(time) => {
                    if previous.is_some_and(|x| x == time) {
                        continue;
                    }
                    previous = Some(time);
                    if !send_json(&mut session, &time).await {
                        debug!("ws connection closed");
                        break;
                    }
                }
                Err(err) => {
                    warn!(?err, "recv failed");
                    break;
                }
            }
        }
    });

    Ok(response)
}

#[put("/contests")]
async fn update_contest(
    data: web::Data<AppData>,
    create_runs: web::Json<ContestState>,
    req: HttpRequest,
) -> impl Responder {
    let contest_key = match data.server_api_key() {
        Some(key) => key,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if req
        .headers()
        .get(API_KEY)
        .is_none_or(|k| k.as_bytes() != contest_key.as_bytes())
    {
        return HttpResponse::Unauthorized().finish();
    };

    let contest_state = create_runs.into_inner();

    match data.update_runs_from_data(contest_state).await {
        Ok(()) => HttpResponse::Created().finish(),
        Err(e) => {
            tracing::error!(?e, "failed updating runs from data");

            HttpResponse::InternalServerError().finish()
        }
    }
}
