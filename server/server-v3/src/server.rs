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
    pub server_api_key: String,
}

pub async fn serve(
    Args {
        port,
        server_api_key,
    }: Args,
) -> color_eyre::eyre::Result<()> {
    let default_timeout = Duration::from_secs(1);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(AppV2::new(
                default_timeout,
                Some(server_api_key.clone()),
            )))
            .service(web::scope("admin").configure(contest_admin::as_service))
            .service(
                web::scope("api").service(endpoints::get_contest_runs::get_contest_runs),
                // .configure(api::configure)
                // .service(get_metrics)
                // .service(remote_control_ws),
            )
        // .service(factory)
        // .service(configure_volumes(volumes.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}
