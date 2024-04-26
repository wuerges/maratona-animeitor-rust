use actix_web::*;
use actix_ws::Closed;
use autometrics::autometrics;
use serde::Deserialize;
use tracing::{debug, warn, Level};

use crate::app_data::AppData;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service((
        get_contest,
        get_timer,
        get_config,
        get_allruns_ws,
        get_allruns_secret,
        get_contest_default,
        get_timer_default,
        get_config_default,
        get_allruns_ws_default,
        get_allruns_secret_default,
    ));
}

#[get("/contest")]
async fn get_contest_default(data: web::Data<AppData>) -> impl Responder {
    get_contest_fn(data, "").await
}

#[get("/files/{sede_config}/contest")]
async fn get_contest(data: web::Data<AppData>, sede_config: web::Path<String>) -> impl Responder {
    get_contest_fn(data, sede_config.as_str()).await
}

#[tracing::instrument(level = Level::DEBUG, skip(data), ret)]
#[autometrics]
async fn get_contest_fn(data: web::Data<AppData>, sede_config: &str) -> impl Responder {
    let db = data.shared_db.lock().await;
    if db.time_file < 0 {
        return HttpResponse::Forbidden().finish();
    }

    match data.config.get(&*sede_config) {
        Some((_, contest, _)) => {
            let result = db.contest_file_begin.clone().filter_sede(&contest.titulo);
            HttpResponse::Ok().json(result)
        }
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/config")]
async fn get_config_default(data: web::Data<AppData>) -> impl Responder {
    get_config_fn(data, "").await
}

#[get("/files/{sede_config}/config")]
async fn get_config(data: web::Data<AppData>, sede_config: web::Path<String>) -> impl Responder {
    get_config_fn(data, sede_config.as_str()).await
}

#[tracing::instrument(level = Level::DEBUG, skip(data), ret)]
#[autometrics]
async fn get_config_fn(data: web::Data<AppData>, sede_config: &str) -> impl Responder {
    let db = data.shared_db.lock().await;
    if db.time_file < 0 {
        return HttpResponse::Forbidden().finish();
    }

    match data.config.get(&*sede_config) {
        Some((config, _, _)) => HttpResponse::Ok().json(config),
        None => HttpResponse::NotFound().finish(),
    }
}

#[derive(Debug, Deserialize)]
struct SecretQuery {
    secret: String,
}

#[tracing::instrument(level = Level::DEBUG, skip(data), ret)]
#[autometrics]
async fn get_allruns_secret_fn(
    data: web::Data<AppData>,
    sede_config: &str,
    query: web::Query<SecretQuery>,
) -> impl Responder {
    let sede = data
        .config
        .get(&*sede_config)
        .map(|(_, _, s)| s.get_sede_by_secret(&query.secret).cloned())
        .flatten();

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

#[get("/files/{sede_config}/allruns_secret")]
async fn get_allruns_secret(
    data: web::Data<AppData>,
    sede_config: web::Path<String>,
    query: web::Query<SecretQuery>,
) -> impl Responder {
    get_allruns_secret_fn(data, sede_config.as_str(), query).await
}

#[get("/allruns_secret")]
async fn get_allruns_secret_default(
    data: web::Data<AppData>,
    query: web::Query<SecretQuery>,
) -> impl Responder {
    get_allruns_secret_fn(data, "", query).await
}

#[get("/files/{sede_config}/allruns_ws")]
async fn get_allruns_ws(
    data: web::Data<AppData>,
    sede_config: web::Path<String>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, Error> {
    get_allruns_ws_fn(data, sede_config.as_str(), req, body).await
}

#[get("/allruns_ws")]
async fn get_allruns_ws_default(
    data: web::Data<AppData>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, Error> {
    get_allruns_ws_fn(data, "", req, body).await
}

#[autometrics]
#[tracing::instrument(level = Level::DEBUG, skip(data, body), ret)]
async fn get_allruns_ws_fn(
    data: web::Data<AppData>,
    sede_config: &str,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, Error> {
    let (response, mut session, _msg_stream) = actix_ws::handle(&req, body)?;
    let mut runs_rx = data.runs_tx.subscribe();

    let sede = data
        .config
        .get(&*sede_config)
        .map(|(_config, contest, _secret)| contest.titulo.clone());

    match sede {
        None => Ok(HttpResponse::Forbidden().finish()),
        Some(sede) => {
            actix_web::rt::spawn(async move {
                loop {
                    match runs_rx.recv().await {
                        Ok(r) => {
                            if sede.team_belongs_str(&r.team_login) {
                                match serde_json::to_string(&r) {
                                    Ok(text) => {
                                        if let Err(Closed) = session.text(text).await {
                                            debug!("ws connection closed");
                                            break;
                                        }
                                    }
                                    Err(err) => warn!(?err, "failed serializing run"),
                                }
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

#[get("/files/{sede_config}/timer")]
async fn get_timer(
    data: web::Data<AppData>,
    _sede_config: web::Path<String>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, Error> {
    get_timer_fn(data, req, body).await
}

#[get("/timer")]
async fn get_timer_default(
    data: web::Data<AppData>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, Error> {
    get_timer_fn(data, req, body).await
}

#[autometrics]
#[tracing::instrument(level = Level::DEBUG, skip(data, body), ret)]
async fn get_timer_fn(
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

                    match serde_json::to_string(&time) {
                        Ok(text) => {
                            if let Err(Closed) = session.text(text).await {
                                debug!("ws connection closed");
                                break;
                            }
                        }
                        Err(err) => warn!(?err, "failed serializing time"),
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
