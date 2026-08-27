mod api;
pub mod metrics;
mod remote_control;

use actix_cors::Cors;
use actix_web::*;

use metrics::get_metrics;
use remote_control::remote_control_ws;
use service::app_data::AppData;
use service::http::load_rustls_config;
use service::volume::Volume;
use service::{
    app_config::AppConfig,
    errors::ServiceResult,
    http::{HttpConfig, HttpTlsConfig},
};
use tracing_actix_web::TracingLogger;

fn configure_volumes(volumes: Vec<Volume>) -> Vec<actix_files::Files> {
    volumes
        .into_iter()
        .map(|Volume { folder, path }| {
            actix_files::Files::new(&path, &folder).index_file("index.html")
        })
        .collect()
}

pub async fn serve_config(
    AppConfig {
        config,
        boca_url,
        server_config: HttpConfig { port, tls },
        volumes,
        server_api_key,
    }: AppConfig,
) -> ServiceResult<()> {
    let data = AppData::new(config, boca_url, server_api_key);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(data.clone()))
            .service(
                web::scope("api")
                    .configure(api::configure)
                    .service(get_metrics)
                    .service(remote_control_ws),
            )
            .service(configure_volumes(volumes.clone()))
    })
    .bind(("0.0.0.0", port))?;

    let server = match tls {
        Some(HttpTlsConfig { cert, key, port: tls_port }) => {
            let tls_config = load_rustls_config(&cert, &key)?;
            server.bind_rustls_0_23(("0.0.0.0", tls_port), tls_config)?
        }
        None => server,
    };

    server.run().await?;

    Ok(())
}
