use axum::{http::StatusCode, response::Response};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(crate) struct About {
    /// Server version from the workspace Cargo.toml.
    version: &'static str,
    /// First 12 hexadecimal characters of the build revision, or null if unavailable.
    #[schema(required = true)]
    commit: Option<&'static str>,
}

pub(crate) async fn get_about() -> Response {
    let mut response = crate::envelope::data_json(
        About {
            version: env!("CARGO_PKG_VERSION"),
            commit: option_env!("ANIMEITOR_BUILD_COMMIT"),
        },
        StatusCode::OK,
    );
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}
