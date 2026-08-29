use autometrics::prometheus_exporter;
use axum::http::StatusCode;

pub fn setup() {
    prometheus_exporter::init();
}

pub async fn get_metrics() -> (StatusCode, String) {
    match prometheus_exporter::encode_to_string() {
        Ok(string) => (StatusCode::OK, string),
        Err(err) => {
            tracing::error!(?err, "metrics");

            (StatusCode::INTERNAL_SERVER_ERROR, String::new())
        }
    }
}
