#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

use std::sync::Arc;
use tokio;
use tokio::{spawn, sync::Mutex};

pub mod models; 
pub mod schema;
pub mod helpers;
pub mod errors;

use lib_server::dataio::*;

#[derive(Copy, Clone, Debug)]
pub struct Params {
    pub contest_number: i32,
    pub site_number: i32,
    // pub connection: &'a PgConnection,
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

async fn update_runs(runs: Arc<Mutex<DB>>, params: Params) -> Result<(), ContestIOError> {

    let connection = establish_connection();

    let contest_data = helpers::get_contest_file(&params, &connection);
    let runs_data = helpers::get_all_runs(&params, &connection);

    let time_data = contest_data.current_time;

    let mut db = runs.lock().await;
    db.refresh_db(time_data, contest_data, runs_data)?;

    Ok(())
}


pub fn spawn_db_update(params: Params) -> Arc<Mutex<DB>> {
    let shared_db = Arc::new(Mutex::new(DB::empty()));
    let cloned_db = shared_db.clone();
    spawn(async move {
        let dur = tokio::time::Duration::new(30, 0);
        let mut interval = tokio::time::interval(dur);
        loop {
            interval.tick().await;
            let r = update_runs(cloned_db.clone(), params).await;
            match r {
                Ok(_) => (),
                Err(e) => eprintln!("Error updating run: {}", e),
            }
        }
    });
    shared_db
}

pub async fn serve_simple_contest(server_port : u16, secret : &String, params: Params) {

    let shared_db = spawn_db_update(params);
    lib_server::serve_simple_contest_assets(shared_db, server_port, secret).await
}

