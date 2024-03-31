use actix_web::*;
use actix_ws::Closed;
use autometrics::autometrics;
use tracing::{debug, warn, Level};

use crate::app_data::AppData;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service((get_contest, get_timer, get_config));
}

#[get("/files/{sede_config}/contest")]
#[autometrics]
#[tracing::instrument(level = Level::DEBUG, skip(data), ret)]
async fn get_contest(data: web::Data<AppData>, sede_config: web::Path<String>) -> impl Responder {
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

#[get("/files/{sede_config}/config")]
#[autometrics]
#[tracing::instrument(level = Level::DEBUG, skip(data), ret)]
async fn get_config(data: web::Data<AppData>, sede_config: web::Path<String>) -> impl Responder {
    let db = data.shared_db.lock().await;
    if db.time_file < 0 {
        return HttpResponse::Forbidden().finish();
    }

    match data.config.get(&*sede_config) {
        Some((config, _, _)) => HttpResponse::Ok().json(config),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/files/{sede_config}/timer")]
#[autometrics]
#[tracing::instrument(level = Level::DEBUG, skip(data, body), ret)]
async fn get_timer(
    data: web::Data<AppData>,
    _sede_config: web::Path<String>,
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
