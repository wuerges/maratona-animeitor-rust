mod api;
mod app_data;
mod endpoints;
pub mod metrics;
mod remote_control;
mod volumes;

use std::{collections::HashMap, fs::File, io::BufReader, path::Path, sync::Arc};

use actix_cors::Cors;
use actix_web::*;
use app_data::AppData;
use rustls::ServerConfig;
use tokio::sync::broadcast;

use metrics::get_metrics;
use remote_control::remote_control_ws;
use service::DB;
use service::dbupdate_v2::db_update_loop;
use service::membroadcast;
use service::{app_config::AppConfig, errors::ServiceResult, http::{HttpConfig, HttpTlsConfig}};
use tokio::sync::Mutex;
use tracing_actix_web::TracingLogger;
use volumes::configure_volumes;

fn load_rustls_config(cert: &Path, key: &Path) -> ServiceResult<ServerConfig> {
    let certs = rustls_pemfile::certs(&mut BufReader::new(File::open(cert)?))
        .collect::<std::io::Result<Vec<_>>>()?;
    let key = rustls_pemfile::private_key(&mut BufReader::new(File::open(key)?))?
        .ok_or_else(|| std::io::Error::other("no private key found in PEM file"))?;
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(std::io::Error::other)
        .map_err(Into::into)
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
    let config = Arc::new(config);

    let shared_db = Arc::new(Mutex::new(DB::empty()));
    let (runs_tx, _) = membroadcast::channel(1000000);
    let (time_tx, _) = broadcast::channel(1000000);

    let remote_control = Arc::new(Mutex::new(HashMap::new()));

    if let Some(url) = boca_url {
        let _update = tokio::task::spawn(db_update_loop(
            url.clone(),
            shared_db.clone(),
            runs_tx.clone(),
            time_tx.clone(),
        ));
    }

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(AppData {
                shared_db: shared_db.clone(),
                runs_tx: runs_tx.clone(),
                time_tx: time_tx.clone(),
                config: config.clone(),
                remote_control: remote_control.clone(),
                server_api_key: server_api_key.clone(),
            }))
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
