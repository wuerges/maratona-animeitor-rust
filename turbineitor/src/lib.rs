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
use tokio::{spawn, sync::Mutex};

use r2d2;

pub mod auth;
pub mod errors;
pub mod helpers;
pub mod models;
pub mod schema;
pub mod server;

use ::server as dserver;

use crate::errors::Error;

use warp::reject::custom;
use warp::Filter;

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

async fn load_data_from_sql(params: Arc<Params>) -> (i64, data::ContestFile, data::RunsFile) {
    load_data_from_sql_maybe(params)
        .await
        .expect("should have loaded data from SQL")
}

async fn serve_sign(
    data: HashMap<String, String>,
    params: Arc<Params>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let login = data
        .get("login")
        .ok_or(warp::reject::custom(Error::empty("login")))?;
    let pass = data
        .get("password")
        .ok_or(warp::reject::custom(Error::empty("password")))?;

    let u = helpers::check_password(&login, &pass, &params).map_err(custom)?;

    auth::sign_user_key(u, params.secret.as_ref()).map_err(custom)
}

pub async fn serve_turbinator_data(server_port: u16, params: Arc<Params>) {
    let params_sign = params.clone();

    let (shared_db, runs_tx) =
        dserver::spawn_db_update_f(move || load_data_from_sql(params.clone()));

    let sign_route = warp::post()
        .and(warp::path("sign"))
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        .and_then(move |m| serve_sign(m, params_sign.clone()));

    let ui_route = warp::get().and(warp::fs::dir("turbineitor/ui"));

    let route_data = dserver::route_contest_public_data(shared_db, runs_tx);

    let routes = sign_route.or(ui_route).or(route_data);

    warp::serve(routes).run(([0, 0, 0, 0], server_port)).await
}
