mod envelope;
pub mod internal;
pub mod memory_files;
pub mod metrics;
pub mod public;
mod remote_control;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::extract::FromRef;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use service::event_store::EventStore;
use service::http::load_rustls_config;
use service::volume::Volume;
use service::{
    app_config::AppConfig,
    errors::ServiceResult,
    http::{HttpConfig, HttpTlsConfig},
};

/// The state shared by all handlers of the server.
#[derive(Clone)]
pub struct AppState {
    pub store: EventStore,
    pub internal_token: Option<String>,
}

impl FromRef<AppState> for EventStore {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

/// Builds the router with the `/api` and `/internal` scopes. The state is
/// provided here: the result is a `Router<()>` ready to serve.
pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/api", public::router())
        .nest("/internal", internal::router())
        .layer(CompressionLayer::new())
        .with_state(state)
}

/// Loads the client assets of a folder into memory once per canonical path,
/// so volumes mounting the same folder (e.g. `-v dist: -v dist:animeitor`)
/// share a single copy.
fn load_assets(
    folder: &str,
    loaded: &mut HashMap<PathBuf, Arc<memory_files::MemoryFiles>>,
) -> Arc<memory_files::MemoryFiles> {
    let canonical = std::fs::canonicalize(folder).unwrap_or_else(|_| PathBuf::from(folder));
    loaded
        .entry(canonical)
        .or_insert_with_key(|dir| Arc::new(memory_files::MemoryFiles::load(dir)))
        .clone()
}

fn volume_router(volume: Volume, loaded: &mut HashMap<PathBuf, Arc<memory_files::MemoryFiles>>) -> Router {
    match volume.path.as_str() {
        "" => {
            // Root mount (landing): anything unmatched by the APIs is served
            // from the client assets held in memory.
            memory_files::router(load_assets(&volume.folder, loaded), "", false)
        }
        "animeitor" => {
            // The client build is an SPA served at /animeitor/{event}/{contest}:
            // unmatched paths fall back to its index.html.
            Router::new().nest(
                "/animeitor",
                memory_files::router(load_assets(&volume.folder, loaded), "/animeitor", true),
            )
        }
        path => Router::new().nest_service(
            &format!("/{path}"),
            ServeDir::new(&volume.folder).append_index_html_on_directories(true),
        ),
    }
}

pub async fn serve_config(
    AppConfig {
        server_config: HttpConfig { port, tls },
        volumes,
        internal_token,
    }: AppConfig,
) -> ServiceResult<()> {
    let state = AppState {
        store: service::event_store::EventStore::new(),
        internal_token,
    };

    let mut app = app(state);
    let mut loaded_assets = HashMap::new();
    for volume in volumes {
        app = app.merge(volume_router(volume, &mut loaded_assets));
    }
    let app = app
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    match tls {
        Some(HttpTlsConfig { cert, key, port: tls_port }) => {
            // Like the old actix server, both listeners stay up: HTTP on
            // `port` and HTTPS on `tls_port`.
            let mut tls_config = load_rustls_config(&cert, &key)?;
            tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

            let handle = axum_server::Handle::new();
            let shutdown_handle = handle.clone();
            tokio::spawn(async move {
                let _ = tokio::signal::ctrl_c().await;
                shutdown_handle.graceful_shutdown(None);
            });

            let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
            let http = axum::serve(listener, app.clone()).with_graceful_shutdown(async {
                let _ = tokio::signal::ctrl_c().await;
            });

            let https = axum_server::tls_rustls::bind_rustls(
                std::net::SocketAddr::from(([0, 0, 0, 0], tls_port)),
                axum_server::tls_rustls::RustlsConfig::from_config(std::sync::Arc::new(
                    tls_config,
                )),
            )
            .handle(handle)
            .serve(app.into_make_service());

            tokio::try_join!(http, https)?;
        }
        None => {
            let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await?;
        }
    }

    Ok(())
}
