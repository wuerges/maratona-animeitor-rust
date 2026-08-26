pub mod contest_signal;
pub mod runs_panel_signal;
pub mod team_signal;

use std::{collections::HashSet, sync::Arc};

use contest_signal::ContestSignal;
use data::{
    annotate_first_solved::annotate_first_solved, configdata::ConfigContest, ContestFile, RunTuple,
    RunsFile,
};
use futures::{channel::mpsc::UnboundedReceiver, StreamExt};
use runs_panel_signal::RunsPanelItemManager;

#[derive(Clone)]
pub struct ContestProvider {
    pub starting_contest: Arc<ContestFile>,
    pub config_contest: Arc<ConfigContest>,
    pub new_contest_signal: Arc<ContestSignal>,
    pub runs_panel_item_manager: Arc<RunsPanelItemManager>,
}

#[derive(Debug)]
pub struct Options {
    pub ready_chunk_capacity: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            ready_chunk_capacity: 100_000,
            // ready_chunk_capacity: 1,
        }
    }
}

pub async fn provide_contest(
    fetch_contest: impl Future<Output = ContestFile>,
    fetch_config: impl Future<Output = ConfigContest>,
) -> ContestProvider {
    let original_contest_file = fetch_contest.await;
    let config = fetch_config.await;
    let original_contest_file = original_contest_file.filter_sede(&config.titulo.into_sede());
    let starting_contest = original_contest_file.clone();

    log::info!("fetched original contest");

    let new_contest_signal = Arc::new(ContestSignal::new(&original_contest_file));
    let runs_panel_item_manager = Arc::new(RunsPanelItemManager::new());

    log::info!("provided contest");
    ContestProvider {
        starting_contest: Arc::new(starting_contest),
        config_contest: Arc::new(config),
        new_contest_signal,
        runs_panel_item_manager,
    }
}

pub async fn poll_runs<F: Future<Output = ()>>(
    starting_contest: Arc<ContestFile>,
    runs_stream: UnboundedReceiver<RunTuple>,
    new_contest_signal: Arc<ContestSignal>,
    runs_panel_item_manager: Arc<RunsPanelItemManager>,
    options: Options,
    sleep: impl Fn() -> F,
) {
    let Options {
        ready_chunk_capacity,
    } = options;

    let mut running_contest = (*starting_contest).clone();
    let mut runs_file = RunsFile::empty();
    let mut solved = HashSet::new();
    let mut runs_stream = runs_stream.ready_chunks(ready_chunk_capacity);

    loop {
        sleep().await;
        // get a new batch of runs
        let next_batch = runs_stream.next().await;
        let size = next_batch.as_ref().map(|v| v.len()).unwrap_or_default();
        log::info!("read next {size:?} runs");

        if let Some(mut next_batch) = next_batch {
            annotate_first_solved(&mut solved, next_batch.iter_mut());
            let mut fresh_runs = vec![];
            for run_tuple in next_batch {
                if runs_file.refresh_1(&run_tuple) {
                    fresh_runs.push(run_tuple);
                }
            }

            if !fresh_runs.is_empty() {
                for r in fresh_runs.iter() {
                    running_contest.apply_run(r);
                }
                running_contest.recalculate_placement();

                for r in fresh_runs.iter() {
                    if let Ok(panel_item) = running_contest.build_panel_item(r) {
                        runs_panel_item_manager.push(panel_item)
                    }
                }

                new_contest_signal.update_tuples(&fresh_runs, &running_contest);
            }
        }
    }
}
