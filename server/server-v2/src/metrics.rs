use actix_web::{get, HttpResponse, Responder};
use autometrics::prometheus_exporter;

pub fn setup() {
    prometheus_exporter::init();
}

#[get("/metrics")]
pub async fn get_metrics() -> impl Responder {
    match prometheus_exporter::encode_to_string() {
        Ok(string) => HttpResponse::Ok().body(string),
        Err(err) => {
            tracing::error!(?err, "metrics");

            HttpResponse::InternalServerError().finish()
        }
    }
}
