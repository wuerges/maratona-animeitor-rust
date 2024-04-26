mod api;
mod app_data;
mod volumes;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::*;
use app_data::AppData;

use service::{
    app_config::AppConfig, dbupdate_v2::spawn_db_update, errors::ServiceResult, http::HttpConfig,
};
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
    let (shared_db, runs_tx, time_tx) = spawn_db_update(&boca_url)?;
    let config = Arc::new(config);

    Ok(HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(AppData {
                shared_db: shared_db.clone(),
                runs_tx: runs_tx.clone(),
                time_tx: time_tx.clone(),
                config: config.clone(),
            }))
            .service(web::scope("api").configure(api::configure))
            .service(configure_volumes(volumes.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?)
}
