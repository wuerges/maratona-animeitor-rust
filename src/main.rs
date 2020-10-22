use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;
use maratona_animeitor_rust::data::*;

#[tokio::main]
async fn main() {

    let runs = Arc::new(
        Mutex::new(
            RunsPanel::from_file("test/sample/runs").unwrap()
        ));
    let runs = warp::any().map(move || Arc::clone(&runs));

    let routes = warp::path("runs").and(runs.clone()).and_then(serve_runs);


    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn serve_runs(runs : Arc<Mutex<RunsPanel>>) 
-> Result<impl warp::Reply, warp::Rejection> {
    let db = runs.lock().await;
    let r = serde_json::to_string(&*db.latest_n(10)).unwrap();
    Ok(r)
}