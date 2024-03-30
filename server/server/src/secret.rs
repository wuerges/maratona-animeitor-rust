use std::sync::Arc;

use autometrics::autometrics;
use data::configdata::Secret;
use serde::Deserialize;
use service::DB;
use tokio::sync::Mutex;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection};

use crate::errors::Error;

pub fn serve_all_runs_secret(runs: Arc<Mutex<DB>>, secrets: Arc<Secret>) -> BoxedFilter<(String,)> {
    warp::any()
        .and(warp::query::<SecretQuery>())
        .and_then(move |query| serve_all_runs_secret_filter(runs.clone(), secrets.clone(), query))
        .boxed()
}

#[derive(Deserialize, Debug)]
struct SecretQuery {
    secret: Option<String>,
}

#[autometrics]
#[tracing::instrument(skip(runs, secrets))]
async fn serve_all_runs_secret_filter(
    runs: Arc<Mutex<DB>>,
    secrets: Arc<Secret>,
    query: SecretQuery,
) -> Result<String, Rejection> {
    Ok(serve_all_runs_secret_service(runs, secrets, query).await?)
}

#[tracing::instrument(skip(runs, secrets), err)]
async fn serve_all_runs_secret_service(
    runs: Arc<Mutex<DB>>,
    secrets: Arc<Secret>,
    query: SecretQuery,
) -> Result<String, Error> {
    match query
        .secret
        .and_then(|secret| secrets.get_sede_by_secret(&secret))
    {
        Some(sede) => {
            let db = runs.lock().await;
            tracing::info!("valid secret");
            Ok(serde_json::to_string(
                &db.run_file_secret.filter_sede(sede),
            )?)
        }
        None => {
            tracing::error!("invalid secret");
            Err(Error::InvalidSecret)
        }
    }
}
