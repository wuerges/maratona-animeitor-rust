mod envelope;
pub mod internal;
pub mod metrics;
pub mod public;
mod remote_control;

use actix_cors::Cors;
use actix_web::*;

use metrics::get_metrics;
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
            let files = actix_files::Files::new(&path, &folder).index_file("index.html");
            // The client build is an SPA served at /animeitor/{event}/{contest}:
            // unmatched paths fall back to its index.html.
            if path == "animeitor" {
                let folder = folder.clone();
                files.default_handler(move |req: actix_web::dev::ServiceRequest| {
                    let folder = folder.clone();
                    async move {
                        let file = actix_files::NamedFile::open(format!("{folder}/index.html"))
                            .map_err(actix_web::error::ErrorInternalServerError)?;
                        let response = file.into_response(req.request());
                        Ok(req.into_response(response))
                    }
                })
            } else {
                files
            }
        })
        .collect()
}

pub async fn serve_config(
    AppConfig {
        server_config: HttpConfig { port, tls },
        volumes,
        internal_token,
    }: AppConfig,
) -> ServiceResult<()> {
    let event_store = service::event_store::EventStore::new();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(event_store.clone()))
            .app_data(web::Data::new(internal::InternalToken(
                internal_token.clone(),
            )))
            .service(
                web::scope("api")
                    .configure(public::configure)
                    .service(get_metrics),
            )
            .service(web::scope("internal").configure(internal::configure))
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
