use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use crate::errors;

pub fn setup() {
    metrics_prometheus::install();
}

async fn encode_metrics() -> Result<String, Rejection> {
    Ok(prometheus::TextEncoder::new()
        .encode_to_string(&prometheus::default_registry().gather())
        .map_err(errors::Error::Prometheus)?)
}

pub(crate) fn route_metrics() -> BoxedFilter<(impl Reply,)> {
    warp::path("metrics").and_then(encode_metrics).boxed()
}
