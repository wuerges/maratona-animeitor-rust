use crate::DB;
use crate::errors::{CResult, Error};

use std::future::Future;
use std::sync::Arc;

use tokio::sync::broadcast;
use tokio::{spawn, sync::Mutex};


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
            tx.send(r.clone()).map_err(|e| Error::DBRefreshError(r, format!("{:?}", e)))?;
        }
    }
    Ok(())
}

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
                    let result = update_runs_from_data(data_ok, cloned_db.clone(), tx.clone()).await;
                    match result {
                        Ok(()) => (),
                        Err(error) => eprintln!("Retrying after error updating runs: \n{}", error),
                    }
                }
                Err(error) => {
                    eprintln!("Retrying after error loading data: \n{}", error);
                }
            }
        }
    });
    (shared_db, tx2)
}