use std::sync::Arc;

use actix_web::*;
use data::{RunTuple, TimerData};
use service::{
    app_config::AppConfig, dbupdate_v2::spawn_db_update, errors::ServiceResult, http::HttpConfig,
    membroadcast, DB,
};
use tokio::sync::{broadcast, Mutex};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}

struct Data {
    shared_db: Arc<Mutex<DB>>,
    runs_tx: membroadcast::Sender<RunTuple>,
    time_tx: broadcast::Sender<TimerData>,
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

    Ok(HttpServer::new(move || {
        App::new()
            .app_data(Data {
                shared_db: shared_db.clone(),
                runs_tx: runs_tx.clone(),
                time_tx: time_tx.clone(),
            })
            .service(greet)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await?)
}
