use std::{collections::HashMap, rc::Rc};

use data::{
    configdata::ConfigContest, ContestFile, ProblemView, RunTuple, RunsFile, RunsPanelItem, Score,
    Team,
};
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use itertools::Itertools;
use leptos::{logging::log, *};

use crate::api::{create_config, create_contest, create_runs, ContestQuery};

pub struct ContestProvider {
    pub running_contest: Signal<ContestFile>,
    pub starting_contest: ContestFile,
    pub config_contest: ConfigContest,
    pub panel_items: ReadSignal<Vec<RunsPanelItem>>,
    pub new_contest_signal: Rc<ContestSignal>,
}

pub struct TeamSignal {
    pub login: String,
    pub name: String,
    pub escola: String,
    pub placement: RwSignal<Option<usize>>,
    pub placement_global: RwSignal<Option<usize>>,
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
            placement_global: _,
            problems,
            id: _,
        } = team;

        Self {
            login: login.clone(),
            name: name.clone(),
            escola: escola.clone(),
            placement: create_rw_signal(None),
            placement_global: create_rw_signal(None),
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
        self.placement_global
            .update(|p| *p = Some(team.placement_global));

        for (letter, problem_view) in &self.problems {
            problem_view.update(|v| *v = team.problems.get(letter).map(|p| p.view()))
        }
    }
}

pub struct ContestSignal {
    pub teams: HashMap<String, TeamSignal>,
}

impl ContestSignal {
    fn new(contest_file: &ContestFile) -> Self {
        let letters = data::PROBLEM_LETTERS[..contest_file.number_problems]
            .chars()
            .map(|l| l.to_string())
            .collect::<Vec<_>>();
        ContestSignal {
            teams: contest_file
                .teams
                .iter()
                .map(|(login, team)| (login.clone(), TeamSignal::new(team, &letters)))
                .collect(),
        }
    }

    fn update(&self, runs: &[RunTuple], fresh_contest: &ContestFile) {
        for login in runs.iter().map(|run| &run.team_login).unique() {
            if let Some(team) = fresh_contest.teams.get(login) {
                if let Some(team_signal) = self.teams.get(login) {
                    team_signal.update(team)
                }
            }
        }
    }
}

pub async fn provide_contest(query: ContestQuery) -> ContestProvider {
    let original_contest_file = create_contest(query.clone()).await;
    let config = create_config(query.clone()).await;
    let original_contest_file = original_contest_file.filter_sede(&config.titulo.into_sede());
    let starting_contest = original_contest_file.clone();

    log!("fetched original contest");
    let (contest_signal, set_contest_signal) =
        create_signal::<ContestFile>(original_contest_file.clone());
    let (runs_panel_signal, set_runs_panel_signal) = create_signal::<Vec<RunsPanelItem>>(vec![]);

    let new_contest_signal = Rc::new(ContestSignal::new(&original_contest_file));
    let new_contest_signal_ref = new_contest_signal.clone();

    spawn_local(async move {
        let mut runs_file = RunsFile::empty();

        let mut runs_stream = create_runs(query).ready_chunks(100_000);

        loop {
            TimeoutFuture::new(1_000).await;
            // get a new batch of runs
            let next_batch = runs_stream.next().await;
            let size = next_batch.as_ref().map(|v| v.len()).unwrap_or_default();
            leptos_dom::logging::console_log(&format!("read next {size:?} runs"));

            if let Some(next_batch) = next_batch {
                let mut fresh_runs = vec![];
                for run_tuple in next_batch {
                    if runs_file.refresh_1(&run_tuple) {
                        fresh_runs.push(run_tuple);
                    }
                }

                if !fresh_runs.is_empty() {
                    let mut fresh_contest = original_contest_file.clone();

                    let mut runs = runs_file.sorted();
                    for r in &runs {
                        fresh_contest.apply_run(r);
                    }

                    fresh_contest.recalculate_placement();
                    new_contest_signal.update(&runs, &fresh_contest);

                    runs.reverse();

                    set_runs_panel_signal.set(
                        runs.into_iter()
                            .filter_map(|r| fresh_contest.build_panel_item(&r).ok())
                            .collect(),
                    );
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
        panel_items: runs_panel_signal,
        new_contest_signal: new_contest_signal_ref,
    }
}
