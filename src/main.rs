use warp::Filter;
use maratona_animeitor_rust::*;

use data::*;

#[tokio::main]
async fn main() {

    let runs = RunsPanel::from_file("test/sample/runs").unwrap();



    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(|name| {
            let j = serde_json::to_string(&runs.latest_n(10)).unwrap();
            format!("Hello, {}!", name)
        });

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}