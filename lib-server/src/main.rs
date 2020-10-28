use std::sync::Arc;
use tokio::{sync::Mutex, spawn};
use tokio;
use std::env;
use warp::Filter;
use lib_server::dataio::*;

use hyper::Client;
use hyper_tls::HttpsConnector;

use hyper::body;
use std::io::prelude::*;
use zip;

// use dataio::*;


#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Expected 2 arguments: {:?}", args);
        return;
    }
    let server_port = match args[1].parse() {
        Ok(t) => t,
        Err(e) => panic!("Could not parse port {}", e)
    };
    let url_base = args[2].clone();

    let shared_db = Arc::new(Mutex::new(DB::empty()));

    
    let shared = Arc::clone(&shared_db);
    spawn(async move {
        let dur = tokio::time::Duration::new(30, 0);
        let mut interval = tokio::time::interval(dur);
        loop {
            interval.tick().await;
            let r = update_runs(&url_base, shared.clone()).await;
            match r {
                Ok(_) => (),
                Err(e) => eprintln!("Error updating run: {}", e)
            }
        }        
    });
    
    type Shared = Arc<Mutex<DB>>;
    fn with_db(db: Shared) -> impl Filter<Extract = (Shared,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    let static_assets = warp::path("static").and(warp::fs::dir("static"));
    let runs = warp::path("runs").and(with_db(shared_db.clone())).and_then(serve_runs);
    let scoreboard = warp::path("score").and(with_db(shared_db)).and_then(serve_score);

    let routes = static_assets.or(runs).or(scoreboard);

    println!("Maratona Streimator rodando!");
    println!("-> Placar em http://localhost:{}/static/runPanel.html", server_port);
    println!("-> Runs em http://localhost:{}/static/scoreboard.html", server_port);
    warp::serve(routes)
        .run(([127, 0, 0, 1], server_port))
        .await;
}



async fn read_bytes_from_url(uri : String) -> Result<Vec<u8>, ContestIOError> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let uri = uri.parse()?; //.map_err( |x| ContestIOError::Chain(x))?;

    let resp = client.get(uri).await?;
    let bytes = body::to_bytes(resp.into_body()).await?;
    Ok(bytes.to_vec())
}

fn read_from_zip(zip : &mut zip::ZipArchive<std::io::Cursor<&std::vec::Vec<u8>>>, name: &str) 
-> Result<String, ContestIOError> {

    let mut runs_zip = zip.by_name(name)
        .map_err(|_| ContestIOError::Info("Could not unpack".to_string()))?;
    let mut buffer = Vec::new();
    runs_zip.read_to_end(&mut buffer)?;
    let runs_data = String::from_utf8(buffer)
        .map_err(|_| ContestIOError::Info("Could not parse to UTF8".to_string()))?;
    Ok(runs_data)
}

async fn update_runs(uri : &String, runs : Arc<Mutex<DB>>) -> Result<(), ContestIOError> {

    let zip_data = read_bytes_from_url(uri.clone()).await?;

    let reader = std::io::Cursor::new(&zip_data);
    let mut zip = zip::ZipArchive::new(reader)
        .map_err(|_| ContestIOError::Info("Could not open zipfile".to_string()))?;

    let mut db = runs.lock().await;
    {
        let time_data = read_from_zip(&mut zip, "./time")?;
        db.reload_time(time_data)?;
    }
    {
        let contest_data = read_from_zip(&mut zip, "./contest")?;
        db.reload_contest(&contest_data)?;
    }
    {
        let runs_data = read_from_zip(&mut zip, "./runs")?;
        db.reload_runs(&runs_data)?;
    }

    db.recalculate_score()?;

    Ok(())
}

async fn serve_runs(runs : Arc<Mutex<DB>>) 
-> Result<impl warp::Reply, warp::Rejection> {
    let db = runs.lock().await;
    let r = serde_json::to_string(&*db.latest_n(50)).unwrap();
    Ok(r)
}

async fn serve_score(runs : Arc<Mutex<DB>>) 
-> Result<impl warp::Reply, warp::Rejection> {
    let db = runs.lock().await;
    let r = serde_json::to_string(&db.get_scoreboard()).unwrap();
    Ok(r)
}