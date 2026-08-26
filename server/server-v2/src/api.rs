use actix_web::*;
use actix_ws::Closed;
use autometrics::autometrics;
use serde::Deserialize;
use tracing::{Level, debug, warn};

use crate::{app_data::AppData, endpoints};

#[derive(Deserialize, Debug)]
struct ContestQuery {
    pub contest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SecretQuery {
    secret: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service((
        get_contest,
        get_timer,
        get_config,
        get_allruns_ws,
        get_allruns_secret,
        endpoints::update_contest::update_contest,
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
    let db = data.shared_db.lock().await;
    if db.time_file < 0 {
        return HttpResponse::Forbidden().finish();
    }

    match data.config.get(sede_config) {
        Some((_, contest, _)) => {
            let result = db.contest_file_begin.clone().filter_sede(&contest.titulo);
            HttpResponse::Ok().json(result)
        }
        None => HttpResponse::NotFound().finish(),
    }
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
    let db = data.shared_db.lock().await;
    if db.time_file < 0 {
        return HttpResponse::Forbidden().finish();
    }

    match data.config.get(sede_config) {
        Some((config, _, _)) => HttpResponse::Ok().json(config),
        None => HttpResponse::NotFound().finish(),
    }
}

#[tracing::instrument(level = Level::DEBUG, skip(data), ret)]
#[autometrics]
async fn get_allruns_secret_fn(data: &AppData, sede_config: &str, secret: &str) -> HttpResponse {
    let sede = data
        .config
        .get(sede_config)
        .and_then(|(_, _, s)| s.get_sede_by_secret(secret).cloned());

    match sede {
        None => HttpResponse::Forbidden().finish(),
        Some(sede) => {
            let db = data.shared_db.lock().await;
            if db.time_file < 0 {
                HttpResponse::Forbidden().finish()
            } else {
                HttpResponse::Ok().json(db.run_file_secret.filter_sede(&sede))
            }
        }
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
    let mut runs_rx = data.runs_tx.subscribe();

    let sede = data
        .config
        .get(contest.into_inner().contest.unwrap_or_default().as_str())
        .map(|(_config, contest, _secret)| contest.titulo.clone());

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
    let mut time_rx = data.time_tx.subscribe();

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
