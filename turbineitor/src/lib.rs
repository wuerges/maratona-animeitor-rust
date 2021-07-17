#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
// use diesel::prelude::*;
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;

use std::sync::Arc;
use tokio;
use tokio::sync::broadcast;
use tokio::{spawn, sync::Mutex};

use r2d2;

pub mod auth;
pub mod errors;
pub mod helpers;
pub mod models;
pub mod schema;
pub mod server;

use ::server as dserver;
use ::server::errors::CResult;

use crate::errors::Error;

use warp::reject::custom;
use warp::Filter;

use futures::{SinkExt, StreamExt};
use warp::ws::Message;

#[derive(Clone)]
pub struct Params {
    pub contest_number: i32,
    pub site_number: i32,
    pub secret: String,
    pool: Pool,
}
impl Params {
    pub fn conn(&self) -> Result<Connection, Error> {
        Ok(self.pool.get()?)
    }

    pub fn new(contest_number: i32, site_number: i32, secret: String) -> Self {
        Self {
            contest_number,
            site_number,
            secret,
            pool: establish_pool(),
        }
    }
}

type Manager = diesel::r2d2::ConnectionManager<PgConnection>;
type Pool = r2d2::Pool<Manager>;
type Connection = r2d2::PooledConnection<Manager>;

pub fn establish_pool() -> Pool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = Manager::new(database_url);

    Pool::builder().max_size(15).build(manager).unwrap()
}

async fn load_data_from_sql_maybe(
    params: Arc<Params>,
) -> Result<(i64, data::ContestFile, data::RunsFile), errors::Error> {
    let contest_data = helpers::get_contest_file(&params)?;
    let runs_data = helpers::get_all_runs(&params)?;

    let time_data = contest_data.current_time;

    Ok((time_data, contest_data, runs_data))
}

async fn load_data_from_sql(params: Arc<Params>) -> CResult<(i64, data::ContestFile, data::RunsFile)> {
    CResult::Ok(load_data_from_sql_maybe(params)
        .await
        .expect("should have loaded data from SQL"))
}

enum TurbMsg {
    Dummy,
    Login { login: String, pass: String },
}

// async fn spawn_serve_data(
//     tx: broadcast::Sender<TurbMsg>,
//     data: HashMap<String, String>,
//     params: Arc<Params>,
// ) {
// }

async fn serve_sign(ws: warp::ws::WebSocket, params: Arc<Params>) {
    let (mut tx, mut rx) = ws.split();

    if let Some(result) = rx.next().await {
        let msg = result.expect("Websocket error");
        let msg = msg.to_str().expect("Should encode message to text");
        let creds: data::auth::Credentials =
            serde_json::from_str(&msg).expect("Should be able to parse login atempt to json");

        match helpers::check_password(&creds.login, &creds.password, &params) {
            Err(e) => {
                let response = data::turb::Msg::Logout;
                let text =
                    serde_json::to_string(&response).expect("Should convert response to json");
                tx.send(warp::ws::Message::text(text))
                    .await
                    .expect("Should send message to client");
                eprintln!("login failure {:?}", e);
            }
            Ok(_) => {
                let response = data::turb::Msg::Login;
                let text =
                    serde_json::to_string(&response).expect("Should convert response to json");
                tx.send(warp::ws::Message::text(text))
                    .await
                    .expect("Should send message to client");

                let dur = tokio::time::Duration::new(1, 0);
                let mut interval = tokio::time::interval(dur);
                let fut = async move {

                    let _tx2 = tx;
                    let _rx2 = rx;
                    println!("keeping connection open for user {}", creds.login);

                    loop {
                        interval.tick().await;
                    }
                };
                tokio::task::spawn(fut);
            }
        }
    }
}

pub async fn serve_turbinator_data(server_port: u16, params: Arc<Params>) {
    let params_sign = params.clone();

    let (shared_db, runs_tx) =
        dserver::spawn_db_update_f(move || load_data_from_sql(params.clone()));

    let sign_route = warp::path("sign")
        .and(warp::ws())
        .and(warp::any().map(move || params_sign.clone()))
        .map(|ws: warp::ws::Ws, params: Arc<Params>| {
            ws.on_upgrade(move |ws| serve_sign(ws, params.clone()))
        });

    // let all_runs_ws = warp::path("allruns_ws")
    // .and(warp::ws())
    // .and(with_db(shared_db.clone()))
    // .and(warp::any().map(move || tx.subscribe()))
    // .map(|ws: warp::ws::Ws, db, rx| ws.on_upgrade(move |ws| serve_all_runs_ws(ws, db, rx)));

    let ui_route = warp::get().and(warp::fs::dir("turbineitor/ui"));

    let route_data = dserver::route_contest_public_data(shared_db, runs_tx);

    let routes = sign_route.or(ui_route).or(route_data);

    warp::serve(routes).run(([0, 0, 0, 0], server_port)).await
}
