pub(crate) mod contest_signal;
pub mod runs_panel_signal;
pub(crate) mod team_signal;

use std::{collections::HashSet, rc::Rc};

use contest_signal::ContestSignal;
use data::{
    annotate_first_solved::annotate_first_solved, configdata::ConfigContest, ContestFile, RunsFile,
};
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use leptos::{logging::log, *};
use runs_panel_signal::RunsPanelItemManager;

use crate::api::{create_config, create_contest, create_runs, ContestQuery};

pub struct ContestProvider {
    pub running_contest: Signal<ContestFile>,
    pub starting_contest: ContestFile,
    pub config_contest: ConfigContest,
    pub new_contest_signal: Rc<ContestSignal>,
    pub runs_panel_item_manager: Rc<RunsPanelItemManager>,
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

pub async fn provide_contest(query: ContestQuery) -> ContestProvider {
    let Options {
        ready_chunk_capacity,
    } = Options::default();

    let original_contest_file = create_contest(query.clone()).await;
    let config = create_config(query.clone()).await;
    let original_contest_file = original_contest_file.filter_sede(&config.titulo.into_sede());
    let starting_contest = original_contest_file.clone();

    log!("fetched original contest");
    let (contest_signal, set_contest_signal) =
        create_signal::<ContestFile>(original_contest_file.clone());

    let new_contest_signal = Rc::new(ContestSignal::new(&original_contest_file));
    let new_contest_signal_ref = new_contest_signal.clone();
    let runs_panel_item_manager = Rc::new(RunsPanelItemManager::new());
    let runs_panel_item_manager_ref = runs_panel_item_manager.clone();

    {
        let mut starting_contest = contest_signal.get_untracked();
        starting_contest.recalculate_placement();
        new_contest_signal.update([].into_iter(), &starting_contest);
    }

    spawn_local(async move {
        let mut runs_file = RunsFile::empty();
        let mut solved = HashSet::new();

        let mut runs_stream = create_runs(query).ready_chunks(ready_chunk_capacity);

        loop {
            TimeoutFuture::new(1_000).await;
            // get a new batch of runs
            let next_batch = runs_stream.next().await;
            let size = next_batch.as_ref().map(|v| v.len()).unwrap_or_default();
            leptos_dom::logging::console_log(&format!("read next {size:?} runs"));

            if let Some(mut next_batch) = next_batch {
                annotate_first_solved(&mut solved, next_batch.iter_mut());
                let mut fresh_runs = vec![];
                for run_tuple in next_batch {
                    if runs_file.refresh_1(&run_tuple) {
                        fresh_runs.push(run_tuple);
                    }
                }

                if !fresh_runs.is_empty() {
                    let mut fresh_contest = contest_signal.get_untracked();

                    let runs = runs_file.sorted();

                    let position = RunsPanelItemManager::position_in_last_submissions(&fresh_runs);

                    for (i, r) in fresh_runs.iter().enumerate() {
                        fresh_contest.apply_run(r);
                        if i >= position {
                            fresh_contest.recalculate_placement();
                        }
                        if let Some(panel_item) = fresh_contest.build_panel_item(&r).ok() {
                            runs_panel_item_manager.push(panel_item)
                        }
                    }
                    fresh_contest.recalculate_placement();

                    new_contest_signal.update_tuples(&runs, &fresh_contest);

                    set_contest_signal.set(fresh_contest);
                }
            }
        }
    });

    log!("provided contest");
    ContestProvider {
        running_contest: contest_signal.into(),
        starting_contest,
        config_contest: config,
        new_contest_signal: new_contest_signal_ref,
        runs_panel_item_manager: runs_panel_item_manager_ref,
    }
}
