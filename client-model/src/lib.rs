pub mod annotate_first_solved;
pub mod contest_signal;
pub mod letters;
pub mod path;
pub mod revelation;
pub mod runs_panel_signal;
pub mod scoring;
pub mod sede_ext;
pub mod team_signal;
pub mod timer_ext;

pub use letters::problem_letters;
pub use revelation::RevelationDriver;
pub use scoring::{
    ContestError, ContestFileExt, ProblemExt, ProblemView, RunsPanelItem, Score, TeamExt,
};
pub use sede_ext::{Color, SedeExt};
pub use timer_ext::TimerDataExt;

use std::{collections::HashSet, sync::Arc};

use contest_signal::ContestSignal;
use data::{ContestFile, RunTuple, RunsFile, configdata::ConfigContest};
use futures::{StreamExt, channel::mpsc::UnboundedReceiver};
use runs_panel_signal::RunsPanelItemManager;

use crate::annotate_first_solved::annotate_first_solved;

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
    let mut solved = HashSet::new();
    let mut runs_file = RunsFile::empty();
    let mut runs_stream = runs_stream.ready_chunks(ready_chunk_capacity);

    loop {
        sleep().await;
        // get a new batch of runs
        let next_batch = runs_stream.next().await;
        let size = next_batch.as_ref().map(|v| v.len()).unwrap_or_default();
        log::info!("read next {size:?} runs");

        if let Some(next_batch) = next_batch {
            let (mut fresh_runs, corrected) = refresh_live_runs(&mut runs_file, next_batch);

            if !fresh_runs.is_empty() {
                if corrected {
                    // Replacing an existing result can undo a solve or penalty.
                    running_contest = rebuild_live_contest(&starting_contest, &runs_file);
                    solved = runs_file
                        .sorted()
                        .into_iter()
                        .filter_map(|run| {
                            matches!(run.answer, data::Answer::Yes { .. }).then_some(run.prob)
                        })
                        .collect();
                } else {
                    annotate_first_solved(&mut solved, fresh_runs.iter_mut());
                    for run in &fresh_runs {
                        running_contest.apply_run(run);
                    }
                    running_contest.recalculate_placement();
                }

                for r in fresh_runs.iter() {
                    if let Ok(panel_item) = running_contest.build_panel_item(r) {
                        runs_panel_item_manager.push(panel_item)
                    }
                }

                if corrected {
                    new_contest_signal.update(
                        starting_contest.teams.keys().map(String::as_str),
                        &running_contest,
                    );
                } else {
                    new_contest_signal.update_tuples(&fresh_runs, &running_contest);
                }
            }
        }
    }
}

// Keep raw server results in the cache; first-solve annotations are display state.
fn refresh_live_runs(runs: &mut RunsFile, batch: Vec<RunTuple>) -> (Vec<RunTuple>, bool) {
    let mut fresh = Vec::new();
    let mut corrected = false;
    for run in batch {
        let previous_len = runs.len();
        if runs.refresh_1(&run) {
            corrected |= runs.len() == previous_len;
            fresh.push(run);
        }
    }
    (fresh, corrected)
}

fn rebuild_live_contest(starting: &ContestFile, runs: &RunsFile) -> ContestFile {
    let mut contest = starting.clone();
    let mut runs = runs.sorted();
    for run in &mut runs {
        if let data::Answer::Yes { is_first, .. } = &mut run.answer {
            *is_first = false;
        }
    }
    annotate_first_solved(&mut HashSet::new(), runs.iter_mut());
    for run in &runs {
        contest.apply_run(run);
    }
    contest.recalculate_placement();
    contest
}

#[cfg(test)]
mod live_scoring_tests {
    use super::*;
    use data::{Answer, Team};

    #[test]
    fn server_masking_correction_undoes_solve_and_penalty() {
        let starting = ContestFile {
            contest_name: "test".into(),
            teams: std::collections::BTreeMap::from([(
                "t".into(),
                Team {
                    login: "t".into(),
                    escola: "School".into(),
                    name: "Team".into(),
                    placement: 1,
                    placement_global: 1,
                    problems: Default::default(),
                    id: 0,
                },
            )]),
            current_time: 200,
            maximum_time: 300,
            score_freeze_time: 100,
            penalty_per_wrong_answer: 1200,
            number_problems: 1,
        };
        let mut runs = RunsFile::empty();
        let mut run = RunTuple {
            id: 1,
            order: 1,
            time: 100,
            team_login: "t".into(),
            prob: "A".parse().unwrap(),
            answer: Answer::No { run_id: 1 },
        };
        let (fresh, corrected) = refresh_live_runs(&mut runs, vec![run.clone()]);
        assert_eq!(fresh.len(), 1);
        assert!(!corrected, "new submissions use incremental scoring");
        let (fresh, corrected) = refresh_live_runs(&mut runs, vec![run.clone()]);
        assert!(fresh.is_empty());
        assert!(!corrected, "identical replay must not rebuild");
        run.id = 2;
        run.order = 2;
        run.answer = Answer::Yes {
            time: 110,
            is_first: false,
            run_id: 2,
        };
        runs.refresh_1(&run);
        let judged = rebuild_live_contest(&starting, &runs);
        let letter = "A".parse().unwrap();
        assert!(judged.teams["t"].problems[&letter].solved);
        assert_eq!(judged.teams["t"].problems[&letter].penalty, 1310);
        for id in [1, 2] {
            run.id = id;
            run.answer = Answer::Wait { run_id: id };
            let (fresh, corrected) = refresh_live_runs(&mut runs, vec![run.clone()]);
            assert_eq!(fresh.len(), 1);
            assert!(corrected, "replaced results must rebuild");
        }
        let masked = rebuild_live_contest(&starting, &runs);
        let problem = &masked.teams["t"].problems[&letter];
        assert!(!problem.solved);
        assert!(!problem.solved_first);
        assert_eq!(problem.penalty, 0);
        assert_eq!(problem.submissions, 0);
        assert_eq!(problem.waits.len(), 2);
    }
}
