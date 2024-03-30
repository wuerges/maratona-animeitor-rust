use crate::config::ServerConfig;
use crate::dbupdate::spawn_db_update;
use crate::membroadcast;
use crate::metrics::route_metrics;
use crate::or_many::OrMany;
use crate::routes;
use crate::runs;
use crate::secret;
use crate::static_routes::serve_static_routes;
use crate::timer;
use autometrics::autometrics;
use data::configdata::ConfigContest;
use data::configdata::Secret;
use service::volume::Volume;
use warp::Rejection;

use crate::errors::Error as CError;

use service::DB;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::broadcast;
use tokio::sync::Mutex;

use warp::Filter;

fn route_contest_public_data(
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
        .and_then(serve_contest_file);

    let routes = runs.or(all_runs_ws).or(timer).or(contest_file);

    routes.boxed()
}

fn serve_urlbase(
    config_map: HashMap<String, ConfigContest>,
    shared_db: Arc<Mutex<DB>>,
    runs_tx: Arc<membroadcast::Sender<data::RunTuple>>,
    time_tx: broadcast::Sender<data::TimerData>,
    secrets: Arc<Secret>,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let routes = config_map
        .into_iter()
        .map(|(config_path, config)| {
            let config = Arc::new(config);
            let config_file = warp::path("config")
                .and(warp::any().map(move || config.clone()))
                .and_then(serve_contest_config);

            let all_runs_secret = warp::path("allruns_secret").and(secret::serve_all_runs_secret(
                shared_db.clone(),
                secrets.clone(),
            ));

            warp::path(config_path)
                .and(
                    route_contest_public_data(shared_db.clone(), runs_tx.clone(), time_tx.clone())
                        .or(config_file)
                        .or(all_runs_secret),
                )
                .boxed()
        })
        .collect_or();

    warp::path("files").and(routes).boxed()
}

#[autometrics]
async fn serve_runs(runs: Arc<Mutex<DB>>) -> Result<String, Rejection> {
    let db = runs.lock().await;
    Ok(serde_json::to_string(&*db.latest()).map_err(CError::SerializationError)?)
}

#[autometrics]
async fn serve_contest_file(runs: Arc<Mutex<DB>>) -> Result<String, Rejection> {
    let db = runs.lock().await;
    if db.time_file < 0 {
        return Err(warp::reject::not_found());
    }
    Ok(serde_json::to_string(&db.contest_file_begin).map_err(CError::SerializationError)?)
}

#[autometrics]
async fn serve_contest_config(config: Arc<ConfigContest>) -> Result<String, Rejection> {
    Ok(serde_json::to_string(&*config).map_err(CError::SerializationError)?)
}

pub async fn serve_simple_contest(
    config: HashMap<String, ConfigContest>,
    boca_url: String,
    secrets: Secret,
    server_config: ServerConfig,
    volumes: Vec<Volume>,
) {
    let port = server_config.port;

    let cors = warp::cors().allow_any_origin();

    let (shared_db, runs_tx, time_tx) = spawn_db_update(&boca_url);

    let service_routes = warp::path("api").and(serve_urlbase(
        config,
        shared_db,
        runs_tx,
        time_tx,
        secrets.into(),
    ));

    let static_routes = serve_static_routes(volumes);

    let all_routes = service_routes
        .or(route_metrics())
        .or(static_routes)
        .with(cors);

    warp::serve(all_routes).run(([0, 0, 0, 0], port)).await;
}
