use crate::errors::CResult;

use std::sync::Arc;
use std::time::Instant;

use crate::membroadcast;
use metrics::{counter, histogram};
use service::DB;
use tokio::sync::broadcast;
use tokio::{spawn, sync::Mutex};

async fn update_runs_from_data(
    data: (i64, data::ContestFile, data::RunsFile),
    runs: &Arc<Mutex<DB>>,
    runs_tx: &membroadcast::Sender<data::RunTuple>,
    time_tx: &broadcast::Sender<data::TimerData>,
) -> CResult<()> {
    let (time_data, contest_data, runs_data) = data;

    let start = Instant::now();

    let mut db = runs.lock().await;
    let fresh_runs = db.refresh_db(time_data, contest_data, runs_data)?;

    let fresh_runs_count = fresh_runs.len() as u64;
    for r in fresh_runs {
        runs_tx.send_memo(r.clone());
    }

    let delta = start.elapsed();

    time_tx.send(db.timer_data()).ok();
    histogram!("update_runs_from_data_time", delta);
    counter!("update_runs_from_data_fresh_runs", fresh_runs_count);
    Ok(())
}

pub fn spawn_db_update(
    boca_url: &str,
) -> (
    Arc<Mutex<DB>>,
    Arc<membroadcast::Sender<data::RunTuple>>,
    broadcast::Sender<data::TimerData>,
) {
    let shared_db = Arc::new(Mutex::new(DB::empty()));
    let cloned_db = shared_db.clone();
    let (orig_runs_tx, _) = membroadcast::channel(1000000);
    let (time_tx, _) = broadcast::channel(1000000);
    let runs_tx = Arc::new(orig_runs_tx);
    let runs_tx_2 = runs_tx.clone();
    let time_tx_2 = time_tx.clone();

    let boca_url = boca_url.to_owned();
    spawn(async move {
        let dur = tokio::time::Duration::new(1, 0);
        let mut interval = tokio::time::interval(dur);
        loop {
            interval.tick().await;

            let start = Instant::now();

            let data = service::webcast::load_data_from_url_maybe(&boca_url).await;

            let delta = start.elapsed();
            let runs_fetched = data
                .as_ref()
                .map(|(_, _, runs)| runs.len())
                .unwrap_or_default() as u64;

            histogram!("load_data_from_url_time", delta);
            counter!("load_data_from_url_all_new_runs_count", runs_fetched);

            match data {
                Ok(data_ok) => {
                    let result =
                        update_runs_from_data(data_ok, &shared_db, &runs_tx, &time_tx).await;
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
    (cloned_db, runs_tx_2, time_tx_2)
}
