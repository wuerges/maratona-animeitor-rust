pub mod runs_panel_signal;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use data::{
    annotate_first_solved::annotate_first_solved, configdata::ConfigContest, ContestFile,
    ProblemView, RunTuple, Score, Team,
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

pub struct TeamSignal {
    pub login: String,
    pub name: String,
    pub escola: String,
    pub placement_global: RwSignal<usize>,
    pub score: RwSignal<Score>,
    pub problems: HashMap<String, RwSignal<Option<ProblemView>>>,
}

impl TeamSignal {
    fn new(team: &Team, letters: &[String]) -> Self {
        let Team {
            login,
            escola,
            name,
            placement: _,
            placement_global,
            problems,
            id: _,
        } = team;

        Self {
            login: login.clone(),
            name: name.clone(),
            escola: escola.clone(),
            placement_global: create_rw_signal(*placement_global),
            score: create_rw_signal(team.score()),
            problems: letters
                .iter()
                .map(|l| {
                    let view = problems.get(l).map(|p| p.view());
                    (l.clone(), create_rw_signal(view))
                })
                .collect(),
        }
    }

    fn update(&self, team: &Team) {
        let new_score = team.score();
        self.score.update(|x| *x = new_score);
        self.placement_global.update(|p| *p = team.placement_global);

        for (letter, problem_view) in &self.problems {
            problem_view.update(|v| *v = team.problems.get(letter).map(|p| p.view()))
        }
    }
}

pub struct ContestSignal {
    pub teams: HashMap<String, Rc<TeamSignal>>,
}

impl ContestSignal {
    pub fn new(contest_file: &ContestFile) -> Self {
        let letters = data::PROBLEM_LETTERS[..contest_file.number_problems]
            .chars()
            .map(|l| l.to_string())
            .collect::<Vec<_>>();
        ContestSignal {
            teams: contest_file
                .teams
                .iter()
                .map(|(login, team)| (login.clone(), Rc::new(TeamSignal::new(team, &letters))))
                .collect(),
        }
    }

    pub fn update<'a>(
        &self,
        team_logins: impl Iterator<Item = &'a str>,
        fresh_contest: &ContestFile,
    ) {
        let update_set: HashSet<_> = team_logins.collect();

        for team in fresh_contest.teams.values() {
            if let Some(team_signal) = self.teams.get(&team.login) {
                if update_set.contains(team.login.as_str()) {
                    team_signal.update(team);
                } else {
                    team_signal.placement_global.set(team.placement_global)
                }
            }
        }
    }

    fn update_tuples(&self, runs: &[RunTuple], fresh_contest: &ContestFile) {
        self.update(
            runs.iter().map(|run| run.team_login.as_str()),
            fresh_contest,
        )
    }
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
        // let mut runs_file = RunsFile::empty();
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

                if !next_batch.is_empty() {
                    let mut fresh_contest = contest_signal.get_untracked();

                    let position = RunsPanelItemManager::position_in_last_submissions(&next_batch);

                    for (i, r) in next_batch.iter().enumerate() {
                        fresh_contest.apply_run(r);
                        if i >= position {
                            fresh_contest.recalculate_placement();
                            if let Some(panel_item) = fresh_contest.build_panel_item(&r).ok() {
                                runs_panel_item_manager.push(panel_item)
                            }
                        }
                    }
                    fresh_contest.recalculate_placement();

                    new_contest_signal.update_tuples(&next_batch, &fresh_contest);

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
