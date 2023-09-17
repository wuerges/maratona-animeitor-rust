use std::sync::Arc;

use autometrics::autometrics;
use data::configdata::ConfigSecretPatterns;
use serde::Deserialize;
use service::DB;
use tokio::sync::Mutex;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection};

use crate::errors::Error;
use crate::routes::with_db;

pub fn serve_all_runs_secret(
    runs: Arc<Mutex<DB>>,
    secrets: Arc<ConfigSecretPatterns>,
) -> BoxedFilter<(String,)> {
    with_db(runs)
        .and(warp::any().map(move || secrets.clone()))
        .and(warp::query::<SecretQuery>())
        .and_then(serve_all_runs_secret_filter)
        .boxed()
}

#[derive(Deserialize)]
struct SecretQuery {
    secret: Option<String>,
}

#[autometrics]
async fn serve_all_runs_secret_filter(
    runs: Arc<Mutex<DB>>,
    secrets: Arc<ConfigSecretPatterns>,
    query: SecretQuery,
) -> Result<String, Rejection> {
    Ok(serve_all_runs_secret_service(runs, secrets, query).await?)
}

async fn serve_all_runs_secret_service(
    runs: Arc<Mutex<DB>>,
    secrets: Arc<ConfigSecretPatterns>,
    query: SecretQuery,
) -> Result<String, Error> {
    match query.secret.and_then(|secret| secrets.secrets.get(&secret)) {
        Some(sede) => {
            let db = runs.lock().await;
            Ok(serde_json::to_string(
                &db.run_file_secret.filter_sede(sede),
            )?)
        }
        None => Err(Error::InvalidSecret),
    }
}
