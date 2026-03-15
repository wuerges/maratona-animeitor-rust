use std::sync::Arc;
use std::time::Instant;

use crate::errors::ServiceResult;
use crate::{DB, membroadcast, webcast};
use data::RunsFile;
use data::contest_state::ContestState;
use metrics::{counter, histogram};
use tokio::sync::Mutex;
use tokio::sync::broadcast;

pub async fn update_runs_from_data(
    data: ContestState,
    shared_db: &Arc<Mutex<DB>>,
    runs_tx: &membroadcast::Sender<data::RunTuple>,
    time_tx: &broadcast::Sender<data::TimerData>,
) -> ServiceResult<()> {
    let ContestState {
        runs,
        time,
        contest,
    } = data;

    let start = Instant::now();

    let mut db = shared_db.lock().await;
    let fresh_runs = db.refresh_db(time, contest, RunsFile::new(runs))?;

    let fresh_runs_count = fresh_runs.len() as u64;
    for r in fresh_runs {
        runs_tx.send_memo(r.clone());
    }

    let delta = start.elapsed();

    time_tx.send(db.timer_data()).ok();
    histogram!("update_runs_from_data_time").record(delta);
    counter!("update_runs_from_data_fresh_runs").increment(fresh_runs_count);
    Ok(())
}

#[allow(clippy::type_complexity)]
pub async fn db_update_loop(
    boca_url: String,
    shared_db: Arc<Mutex<DB>>,
    runs_tx: membroadcast::Sender<data::RunTuple>,
    time_tx: broadcast::Sender<data::TimerData>,
) -> ServiceResult<()> {
    let dur = tokio::time::Duration::new(1, 0);
    let mut interval = tokio::time::interval(dur);
    loop {
        interval.tick().await;

        let start = Instant::now();

        let data = webcast::load_data_from_url_maybe(&boca_url).await;

        let delta = start.elapsed();
        let runs_fetched = data.as_ref().map(|cs| cs.runs.len()).unwrap_or_default() as u64;

        histogram!("load_data_from_url_time").record(delta);
        counter!("load_data_from_url_all_new_runs_count").increment(runs_fetched);

        match data {
            Ok(contest_state) => {
                let result =
                    update_runs_from_data(contest_state, &shared_db, &runs_tx, &time_tx).await;
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
}
