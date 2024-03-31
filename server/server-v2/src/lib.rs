use std::{collections::HashMap, sync::Arc};

use actix_web::*;
use autometrics::autometrics;
use data::{
    configdata::{ConfigContest, Contest, Secret},
    RunTuple, TimerData,
};
use service::{
    app_config::AppConfig, dbupdate_v2::spawn_db_update, errors::ServiceResult, http::HttpConfig,
    membroadcast, DB,
};
use tokio::sync::{broadcast, Mutex};

#[get("/files/{sede_config}/contest")]
#[autometrics]
async fn serve_contest_file(
    data: web::Data<Data>,
    sede_config: web::Path<String>,
) -> impl Responder {
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

struct Data {
    shared_db: Arc<Mutex<DB>>,
    runs_tx: membroadcast::Sender<RunTuple>,
    time_tx: broadcast::Sender<TimerData>,
    config: Arc<HashMap<String, (ConfigContest, Contest, Secret)>>,
}

pub async fn serve_config(
    AppConfig {
        config,
        boca_url,
        server_config: HttpConfig { port },
        volumes,
    }: AppConfig,
) -> ServiceResult<()> {
    let (shared_db, runs_tx, time_tx) = spawn_db_update(&boca_url)?;
    let config = Arc::new(config);

    Ok(HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Data {
                shared_db: shared_db.clone(),
                runs_tx: runs_tx.clone(),
                time_tx: time_tx.clone(),
                config: config.clone(),
            }))
            .service(serve_contest_file)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?)
}
