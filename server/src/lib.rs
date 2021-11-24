pub mod config;
pub mod dataio;
pub mod errors;

use data::configdata::ConfigContest;

use crate::errors::{CResult, Error};

extern crate html_escape;
extern crate itertools;
extern crate rand;

use crate::dataio::*;

use hyper::Client;
use hyper_tls::HttpsConnector;

use hyper::body;
use std::future::Future;
use std::io::prelude::*;
use std::sync::Arc;
use tokio;
use tokio::sync::broadcast;
use tokio::{spawn, sync::Mutex};
use zip;

use futures::{SinkExt, StreamExt};
use warp::ws::Message;

use warp::Filter;

pub fn spawn_db_update_f<F, Fut>(loader: F) -> (Arc<Mutex<DB>>, broadcast::Sender<data::RunTuple>)
where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future<Output = CResult<(i64, data::ContestFile, data::RunsFile)>> + Send,
{
    let shared_db = Arc::new(Mutex::new(DB::empty()));
    let cloned_db = shared_db.clone();
    let (tx, _) = broadcast::channel(1000000);
    let tx2 = tx.clone();

    spawn(async move {
        let dur = tokio::time::Duration::new(1, 0);
        let mut interval = tokio::time::interval(dur);
        loop {
            interval.tick().await;

            let data = loader();

            match data.await {
                Ok(data_ok) => {
                    let r = update_runs_from_data(data_ok, cloned_db.clone(), tx.clone()).await;
                    match r {
                        Ok(_) => (),
                        Err(e) => eprintln!("Error updating run: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("Error loading data: {}, retrying.", e);
                }
            }
        }
    });
    (shared_db, tx2)
}

fn with_db(
    db: Arc<Mutex<DB>>,
) -> impl Filter<Extract = (Arc<Mutex<DB>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub fn route_contest_public_data(
    shared_db: Arc<Mutex<DB>>,
    tx: broadcast::Sender<data::RunTuple>,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let runs = warp::path("runs")
        .and(with_db(shared_db.clone()))
        .and_then(serve_runs);

    let all_runs_ws = warp::path("allruns_ws")
        .and(warp::ws())
        .and(with_db(shared_db.clone()))
        .and(warp::any().map(move || tx.subscribe()))
        .map(|ws: warp::ws::Ws, db, rx| ws.on_upgrade(move |ws| serve_all_runs_ws(ws, db, rx)));

    let timer = warp::path("timer")
        .and(warp::ws())
        .and(with_db(shared_db.clone()))
        .map(|ws: warp::ws::Ws, db| ws.on_upgrade(move |ws| serve_timer_ws(ws, db)));

    let contest_file = warp::path("contest")
        .and(with_db(shared_db.clone()))
        .and_then(serve_contestfile);

    let routes = runs.or(all_runs_ws).or(timer).or(contest_file);

    routes.boxed()
}

pub fn serve_urlbase(
    config: ConfigContest,
    shared_db: Arc<Mutex<DB>>,
    tx: broadcast::Sender<data::RunTuple>,
    secret: &String,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let config = Arc::new(config);
    let config_file = warp::path("config")
        .and(warp::any().map(move || config.clone()))
        .and_then(serve_contest_config);

    let all_runs_secret = warp::path(format!("allruns_{}", secret))
        .and(with_db(shared_db.clone()))
        .and_then(serve_all_runs_secret);

    route_contest_public_data(shared_db, tx)
        .or(config_file)
        .or(all_runs_secret)
        .boxed()
}

async fn read_bytes_from_path(path: &String) -> CResult<Vec<u8>> {
    read_bytes_from_url(path)
        .await
        .or_else(|_| read_bytes_from_file(path))
}

fn read_bytes_from_file(path: &String) -> CResult<Vec<u8>> {
    Ok(std::fs::read(&path)?)
}

async fn read_bytes_from_url(uri: &String) -> CResult<Vec<u8>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let uri = uri.parse()?;

    let resp = client.get(uri).await?;
    let bytes = body::to_bytes(resp.into_body()).await?;
    Ok(bytes.to_vec())
}

fn try_read_from_zip(
    zip: &mut zip::ZipArchive<std::io::Cursor<&std::vec::Vec<u8>>>,
    name: &str,
) -> CResult<String> {
    let mut runs_zip = zip
        .by_name(name)
        .map_err(|e| Error::Info(format!("Could not unpack file: {} {:?}", name, e)))?;
    let mut buffer = Vec::new();
    runs_zip.read_to_end(&mut buffer)?;
    let runs_data = String::from_utf8(buffer)
        .map_err(|_| Error::Info("Could not parse to UTF8".to_string()))?;
    Ok(runs_data)
}

fn read_from_zip(
    zip: &mut zip::ZipArchive<std::io::Cursor<&std::vec::Vec<u8>>>,
    name: &str,
) -> CResult<String> {
    try_read_from_zip(zip, name)
        .or_else(|_| try_read_from_zip(zip, &format!("./{}", name)))
        .or_else(|_| try_read_from_zip(zip, &format!("./sample/{}", name)))
        .or_else(|_| try_read_from_zip(zip, &format!("sample/{}", name)))
}

pub async fn load_data_from_url_maybe(
    uri: String,
) -> CResult<(i64, data::ContestFile, data::RunsFile)> {
    let zip_data = read_bytes_from_path(&uri).await?;

    let reader = std::io::Cursor::new(&zip_data);
    let mut zip = zip::ZipArchive::new(reader)
        .map_err(|e| Error::Info(format!("Could not open zipfile: {:?}", e)))?;

    let time_data: i64 = read_from_zip(&mut zip, "time")?.parse()?;

    let contest_data = read_from_zip(&mut zip, "contest")?;
    let contest_data = read_contest(&contest_data)?;

    let runs_data = read_from_zip(&mut zip, "runs")?;
    let runs_data = read_runs(&runs_data)?;

    Ok((time_data, contest_data, runs_data))
}

async fn update_runs_from_data(
    data: (i64, data::ContestFile, data::RunsFile),
    runs: Arc<Mutex<DB>>,
    tx: broadcast::Sender<data::RunTuple>,
) -> CResult<()> {
    let (time_data, contest_data, runs_data) = data;

    let mut db = runs.lock().await;
    let fresh_runs = db.refresh_db(time_data, contest_data, runs_data)?;
    if tx.receiver_count() > 0 {
        for r in fresh_runs {
            tx.send(r).expect("Should have sent the messages");
        }
    }
    Ok(())
}

async fn serve_runs(runs: Arc<Mutex<DB>>) -> Result<impl warp::Reply, warp::Rejection> {
    let db = runs.lock().await;
    let r = serde_json::to_string(&*db.latest()).unwrap();
    Ok(r)
}

async fn serve_timer_ws(ws: warp::ws::WebSocket, runs: Arc<Mutex<DB>>) {
    let (mut tx, _) = ws.split();

    let fut = async move {
        let dur = tokio::time::Duration::new(1, 0);
        let mut interval = tokio::time::interval(dur);
        let mut old = data::TimerData::fake();

        loop {
            interval.tick().await;
            let l = runs.lock().await.timer_data();

            if l != old {
                old = l;
                let t = serde_json::to_string(&l).unwrap();
                let m = Message::text(t);
                tx.send(m).await.expect("Error sending");
            }
        }
    };

    tokio::task::spawn(fut);
}

async fn serve_all_runs_ws(
    ws: warp::ws::WebSocket,
    runs: Arc<Mutex<DB>>,
    mut rx: broadcast::Receiver<data::RunTuple>,
) {
    let (mut tx, _) = ws.split();

    let fut = async move {
        {
            let lock = runs.lock().await;

            for r in &lock.all_runs() {
                let t = serde_json::to_string(r).unwrap();
                let m = Message::text(t);
                tx.send(m).await.expect("Error sending");
            }
        }

        loop {
            let r = rx.recv().await.expect("Expected a RunTuple");

            let t = serde_json::to_string(&r).unwrap();
            let m = Message::text(t);

            tx.send(m).await.expect("Should have sent the message");
        }
    };

    tokio::task::spawn(fut);
}

async fn serve_all_runs_secret(runs: Arc<Mutex<DB>>) -> Result<impl warp::Reply, warp::Rejection> {
    let db = runs.lock().await;
    let r = serde_json::to_string(&db.run_file_secret).unwrap();
    Ok(r)
}

async fn serve_contestfile(runs: Arc<Mutex<DB>>) -> Result<impl warp::Reply, warp::Rejection> {
    let db = runs.lock().await;
    let r = serde_json::to_string(&db.contest_file_begin).unwrap();
    Ok(r)
}

async fn serve_contest_config(
    config: Arc<ConfigContest>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let r = serde_json::to_string(&*config).unwrap();
    Ok(r)
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
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub async fn serve_simple_contest(
    config: ConfigContest,
    url_base: String,
    server_port: u16,
    secret: &String,
) {
    serve_simple_contest_f(
        config,
        move || load_data_from_url_maybe(url_base.clone()),
        server_port,
        secret,
    )
    .await
}

pub async fn serve_simple_contest_f<F, Fut>(
    config: ConfigContest,
    f: F,
    server_port: u16,
    secret: &String,
) where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future<Output = CResult<(i64, data::ContestFile, data::RunsFile)>> + Send,
{
    let (shared_db, runs_tx) = spawn_db_update_f(f);
    serve_simple_contest_assets(config, shared_db, runs_tx, server_port, secret).await
}

pub async fn serve_simple_contest_assets(
    config: ConfigContest,
    db: Arc<Mutex<DB>>,
    tx: broadcast::Sender<data::RunTuple>,
    server_port: u16,
    secret: &String,
) {
    let seed_assets = warp::fs::dir("client");

    // let root = warp::path::end()
    //     .map(|| warp::redirect(warp::http::Uri::from_static("/seed/everything2.html")));

    let routes = serve_urlbase(config, db, tx, secret).or(seed_assets);


    warp::serve(routes).run(([0, 0, 0, 0], server_port)).await
}
