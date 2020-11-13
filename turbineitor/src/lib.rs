#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::prelude::*;
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

use lib_server::dataio::*;

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

// pub fn establish_threaded_connection() -> Arc<Mutex<PgConnection>> {
//     Arc::new(Mutex::new(establish_connection()))
// }

// pub fn establish_connection() -> PgConnection {
//     dotenv().ok();

//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
// }

async fn update_runs(
    runs: Arc<Mutex<DB>>,
    params: &Params,
) -> Result<(), Error> {
    let contest_data = helpers::get_contest_file(&params)?;
    let runs_data = helpers::get_all_runs(&params)?;

    let time_data = contest_data.current_time;

    let mut db = runs.lock().await;
    db.refresh_db(time_data, contest_data, runs_data)?;
    Ok(())
}

pub fn spawn_db_update(params: &Params) -> Arc<Mutex<DB>> {
    let shared_db = Arc::new(Mutex::new(DB::empty()));
    let cloned_db = shared_db.clone();
    let params = params.clone();
    spawn(async move {
        let dur = tokio::time::Duration::new(30, 0);
        let mut interval = tokio::time::interval(dur);
        // let params = params.clone();
        // let connection = establish_threaded_connection();
        loop {
            interval.tick().await;
            let r = update_runs(cloned_db.clone(), &params).await;
            match r {
                Ok(_) => (),
                Err(e) => eprintln!("Error updating run: {}", e),
            }
        }
    });
    shared_db
}

pub async fn serve_simple_contest(server_port: u16, params: Params) {
    let shared_db = spawn_db_update(&params);
    lib_server::serve_simple_contest_assets(shared_db, server_port, &params.secret).await
}
