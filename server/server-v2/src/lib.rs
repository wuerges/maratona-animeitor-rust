mod about;
mod envelope;
pub mod internal;
pub mod memory_files;
pub mod metrics;
pub mod openapi;
pub mod public;
mod remote_control;
mod request_logging;
mod resource_limits;
mod transport;
mod websocket_liveness;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

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
    pub public_url: url::Url,
    pub store: EventStore,
    pub internal_tokens:
        std::sync::Arc<std::collections::HashMap<String, service::internal_auth::InternalToken>>,
}

impl FromRef<AppState> for EventStore {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

/// Builds the router with the `/api` and `/internal` scopes. The state is
/// provided here: the result is a `Router<()>` ready to serve.
pub fn app(state: AppState) -> Router {
    let capacity = state.store.limits().max_response_bodies;
    request_logging::layer(resource_limits::responses(api_router(state), capacity))
}

fn api_router(state: AppState) -> Router {
    let limits = state.store.limits().clone();
    Router::new()
        .nest(
            "/api",
            resource_limits::http(
                public::router(),
                limits.max_public_requests,
                limits.request_timeout_seconds,
            ),
        )
        .nest(
            "/internal",
            resource_limits::http(
                internal::router(),
                limits.max_internal_requests,
                limits.request_timeout_seconds,
            ),
        )
        .layer(CompressionLayer::new())
        .with_state(state)
}

/// The internal API must never receive credentials over cleartext HTTP.
async fn reject_internal_over_http(
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    if request.uri().path().starts_with("/internal/") || request.uri().path() == "/internal" {
        return (
            StatusCode::UPGRADE_REQUIRED,
            [(axum::http::header::CONNECTION, "close")],
            "the internal API requires HTTPS",
        )
            .into_response();
    }
    next.run(request).await
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

fn volume_router(
    volume: Volume,
    loaded: &mut HashMap<PathBuf, Arc<memory_files::MemoryFiles>>,
) -> Router {
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
        limits,
        database,
        public_url,
        server_config: HttpConfig { port, tls },
        volumes,
        internal_tokens,
        revelation_salt,
    }: AppConfig,
) -> ServiceResult<()> {
    let database: Arc<dyn service::database::Database> = match database {
        service::database::DatabaseConfig::Memory {} => {
            Arc::new(database_memory::MemoryDatabase::new())
        }
        service::database::DatabaseConfig::Sqlite { path } => {
            Arc::new(database_sqlite::SqliteDatabase::open(path)?)
        }
    };
    limits.validate().map_err(std::io::Error::other)?;
    tracing::info!(?limits, "effective resource limits");
    let state = AppState {
        public_url,
        store: service::event_store::EventStore::with_limits(
            database,
            revelation_salt,
            limits.clone(),
        ),
        internal_tokens: Arc::new(internal_tokens),
    };

    state
        .store
        .initialize()
        .await
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    let mut app = api_router(state);
    let mut loaded_assets = HashMap::new();
    for volume in volumes {
        app = app.merge(volume_router(volume, &mut loaded_assets));
    }
    let app = app.layer(
        CorsLayer::permissive()
            .expose_headers([axum::http::HeaderName::from_static("x-request-id")]),
    );

    let app = resource_limits::responses(app, limits.max_response_bodies);
    let acceptor = transport::Acceptor::new(limits.clone());
    let handle = axum_server::Handle::new();
    let shutdown = handle.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        shutdown.graceful_shutdown(Some(std::time::Duration::from_secs(15)));
    });
    let http_app = request_logging::layer(
        app.clone()
            .layer(middleware::from_fn(reject_internal_over_http)),
    );
    let mut http = axum_server::bind(std::net::SocketAddr::from(([0, 0, 0, 0], port)))
        .acceptor(acceptor.clone())
        .handle(handle.clone());
    transport::configure(http.http_builder(), &limits);
    match tls {
        Some(HttpTlsConfig {
            cert,
            key,
            port: tls_port,
        }) => {
            let mut tls_config = load_rustls_config(&cert, &key)?;
            tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
            let tls_acceptor = axum_server::tls_rustls::RustlsAcceptor::new(
                axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(tls_config)),
            )
            .handshake_timeout(std::time::Duration::from_secs(
                limits.tls_handshake_timeout_seconds,
            ))
            .acceptor(acceptor);
            let mut https = axum_server::bind(std::net::SocketAddr::from(([0, 0, 0, 0], tls_port)))
                .acceptor(tls_acceptor)
                .handle(handle);
            transport::configure(https.http_builder(), &limits);
            tokio::try_join!(
                http.serve(http_app.into_make_service()),
                https.serve(request_logging::layer(app).into_make_service())
            )?;
        }
        None => http.serve(http_app.into_make_service()).await?,
    }

    Ok(())
}

/// Propagate storage errors through the same envelope as domain errors.
#[macro_export]
macro_rules! store_call {
    ($operation:expr) => {
        match $operation {
            Ok(value) => value,
            Err(error) => return $crate::internal::store_error(error),
        }
    };
}
