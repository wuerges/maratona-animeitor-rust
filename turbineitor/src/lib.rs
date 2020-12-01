#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
// use diesel::prelude::*;
use dotenv::dotenv;
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

    pub fn new(contest_number: i32, site_number:i32, secret: String) -> Self {
        Self {
            contest_number,
            site_number,
            secret,
            pool : establish_pool()
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
        .expect("should have loaded data from url")
}


pub async fn serve_simple_contest(server_port: u16, params: Arc<Params>) {
    let secret = params.secret.clone();
    dserver::serve_simple_contest_f(move || load_data_from_sql(params.clone()), server_port, &secret).await
}
