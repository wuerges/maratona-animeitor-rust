mod api;
mod app_data;
pub mod metrics;
mod remote_control;
mod volumes;

use std::{collections::HashMap, sync::Arc};

use actix_cors::Cors;
use actix_web::*;
use app_data::AppData;
use tokio::sync::broadcast;

use metrics::get_metrics;
use remote_control::remote_control_ws;
use service::DB;
use service::dbupdate_v2::db_update;
use service::membroadcast;
use service::{app_config::AppConfig, errors::ServiceResult, http::HttpConfig};
use tokio::sync::Mutex;
use tracing_actix_web::TracingLogger;
use volumes::configure_volumes;

pub async fn serve_config(
    AppConfig {
        config,
        boca_url,
        server_config: HttpConfig { port },
        volumes,
    }: AppConfig,
) -> ServiceResult<()> {
    let config = Arc::new(config);

    let shared_db = Arc::new(Mutex::new(DB::empty()));
    let (runs_tx, _) = membroadcast::channel(1000000);
    let (time_tx, _) = broadcast::channel(1000000);

    let remote_control = Arc::new(Mutex::new(HashMap::new()));

    let _update = tokio::task::spawn(db_update(
        boca_url.clone(),
        shared_db.clone(),
        runs_tx.clone(),
        time_tx.clone(),
    ));

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(AppData {
                shared_db: shared_db.clone(),
                runs_tx: runs_tx.clone(),
                time_tx: time_tx.clone(),
                config: config.clone(),
                remote_control: remote_control.clone(),
            }))
            .service(
                web::scope("api")
                    .configure(api::configure)
                    .service(get_metrics)
                    .service(remote_control_ws),
            )
            .service(configure_volumes(volumes.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}
