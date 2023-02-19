pub mod config;
mod dbupdate;
mod errors;
mod membroadcast;
mod routes;
mod runs;
mod timer;

use data::configdata::ConfigContest;
use futures::TryFutureExt;
use warp::Rejection;

use crate::dbupdate::spawn_db_update_f;
use crate::errors::CResult;
use crate::errors::Error as CError;

extern crate html_escape;
extern crate itertools;
extern crate rand;

use service::DB;

use std::future::Future;
use std::sync::Arc;

use tokio::sync::broadcast;
use tokio::sync::Mutex;

use warp::Filter;

pub fn route_contest_public_data(
    shared_db: Arc<Mutex<DB>>,
    runs_tx: Arc<membroadcast::Sender<data::RunTuple>>,
    time_tx: broadcast::Sender<data::TimerData>,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let runs = warp::path("runs")
        .and(routes::with_db(shared_db.clone()))
        .and_then(serve_runs);

    let all_runs_ws = warp::path("allruns_ws").and(runs::serve_all_runs(runs_tx));

    let timer = warp::path("timer").and(timer::serve_timer(time_tx));

    let contest_file = warp::path("contest")
        .and(routes::with_db(shared_db))
        .and_then(serve_contestfile);

    let routes = runs.or(all_runs_ws).or(timer).or(contest_file);

    routes.boxed()
}

pub fn serve_urlbase(
    config: ConfigContest,
    shared_db: Arc<Mutex<DB>>,
    runs_tx: Arc<membroadcast::Sender<data::RunTuple>>,
    time_tx: broadcast::Sender<data::TimerData>,
    secret: &String,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let config = Arc::new(config);
    let config_file = warp::path("config")
        .and(warp::any().map(move || config.clone()))
        .and_then(serve_contest_config);

    let all_runs_secret = warp::path(format!("allruns_{}", secret))
        .and(routes::with_db(shared_db.clone()))
        .and_then(serve_all_runs_secret);

    route_contest_public_data(shared_db, runs_tx, time_tx)
        .or(config_file)
        .or(all_runs_secret)
        .boxed()
}

async fn serve_runs(runs: Arc<Mutex<DB>>) -> Result<String, Rejection> {
    let db = runs.lock().await;
    Ok(serde_json::to_string(&*db.latest()).map_err(CError::SerializationError)?)
}

async fn serve_all_runs_secret(runs: Arc<Mutex<DB>>) -> Result<String, Rejection> {
    let db = runs.lock().await;
    Ok(serde_json::to_string(&db.run_file_secret).map_err(CError::SerializationError)?)
}

async fn serve_contestfile(runs: Arc<Mutex<DB>>) -> Result<String, Rejection> {
    let db = runs.lock().await;
    if db.time_file < 0 {
        return Err(warp::reject::not_found());
    }
    Ok(serde_json::to_string(&db.contest_file_begin).map_err(CError::SerializationError)?)
}

async fn serve_contest_config(config: Arc<ConfigContest>) -> Result<String, Rejection> {
    Ok(serde_json::to_string(&*config).map_err(CError::SerializationError)?)
}

pub fn random_path_part() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const PASSWORD_LEN: usize = 6;
    let mut rng = rand::thread_rng();
    (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub async fn serve_simple_contest(
    config: ConfigContest,
    url_base: String,
    server_port: u16,
    secret: &String,
    lambda_mode: bool,
) {
    serve_simple_contest_f(
        config,
        move || {
            service::webcast::load_data_from_url_maybe(url_base.clone())
                .map_err(CError::ServiceError)
        },
        server_port,
        secret,
        lambda_mode,
    )
    .await
}

pub async fn serve_simple_contest_f<F, Fut>(
    config: ConfigContest,
    f: F,
    server_port: u16,
    secret: &String,
    lambda_mode: bool,
) where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future<Output = CResult<(i64, data::ContestFile, data::RunsFile)>> + Send,
{
    let (shared_db, runs_tx, time_tx) = spawn_db_update_f(f);
    serve_simple_contest_assets(
        config,
        shared_db,
        runs_tx,
        time_tx,
        server_port,
        secret,
        lambda_mode,
    )
    .await
}

pub async fn serve_simple_contest_assets(
    config: ConfigContest,
    db: Arc<Mutex<DB>>,
    runs_tx: Arc<membroadcast::Sender<data::RunTuple>>,
    time_tx: broadcast::Sender<data::TimerData>,
    server_port: u16,
    secret: &String,
    lambda_mode: bool,
) {
    let services = serve_urlbase(config, db, runs_tx, time_tx, secret);
    let cors = warp::cors().allow_any_origin();

    let services = services.with(cors);

    if lambda_mode {
        warp::serve(services).run(([0, 0, 0, 0], server_port)).await;
    } else {
        let seed_assets = warp::fs::dir("client");
        let routes = services.or(seed_assets);
        warp::serve(routes).run(([0, 0, 0, 0], server_port)).await;
    };
}
