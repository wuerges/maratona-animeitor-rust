mod api;
mod app_data;

use std::sync::Arc;

use actix_web::*;
use app_data::AppData;

use service::{
    app_config::AppConfig, dbupdate_v2::spawn_db_update, errors::ServiceResult, http::HttpConfig,
};

pub async fn serve_config(
    AppConfig {
        config,
        boca_url,
        server_config: HttpConfig { port },
        volumes: _,
    }: AppConfig,
) -> ServiceResult<()> {
    let (shared_db, runs_tx, time_tx) = spawn_db_update(&boca_url)?;
    let config = Arc::new(config);

    Ok(HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppData {
                shared_db: shared_db.clone(),
                runs_tx: runs_tx.clone(),
                time_tx: time_tx.clone(),
                config: config.clone(),
            }))
            .service(web::scope("api").configure(api::configure))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?)
}
