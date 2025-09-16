use std::time::Duration;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use tracing_actix_web::TracingLogger;

use crate::{
    endpoints::{self, contest_admin},
    model::app::AppV2,
};

pub struct Args {
    pub port: u16,
    pub server_api_key: Option<String>,
}

pub async fn serve(
    Args {
        port,
        server_api_key,
    }: Args,
) -> color_eyre::eyre::Result<()> {
    let default_timeout = Duration::from_secs(1);

    let app = AppV2::new(default_timeout, server_api_key.clone());

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(app.clone()))
            .service(web::scope("api/internal").configure(contest_admin::as_service))
            .service(
                web::scope("api").configure(endpoints::as_service),
                // .configure(api::configure)
                // .service(get_metrics)
                // .service(remote_control_ws),
            )
            .wrap(TracingLogger::default())
        // .service(factory)
        // .service(configure_volumes(volumes.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}
