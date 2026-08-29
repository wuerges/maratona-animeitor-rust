mod envelope;
pub mod internal;
pub mod metrics;
pub mod public;
mod remote_control;

use axum::Router;
use axum::extract::FromRef;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
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
        .with_state(state)
}

fn volume_router(Volume { folder, path }: Volume) -> Router {
    let router = Router::new();
    if path.is_empty() {
        // Root mount (landing): anything unmatched by the APIs is served
        // from the static files.
        router.fallback_service(
            ServeDir::new(&folder).append_index_html_on_directories(true),
        )
    } else if path == "animeitor" {
        // The client build is an SPA served at /animeitor/{event}/{contest}:
        // unmatched paths fall back to its index.html.
        router.nest_service(
            &format!("/{path}"),
            ServeDir::new(&folder)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(format!("{folder}/index.html"))),
        )
    } else {
        router.nest_service(
            &format!("/{path}"),
            ServeDir::new(&folder).append_index_html_on_directories(true),
        )
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
    for volume in volumes {
        app = app.merge(volume_router(volume));
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
