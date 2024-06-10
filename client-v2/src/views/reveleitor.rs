use std::rc::Rc;

use data::{configdata::Sede, revelation::RevelationDriver, ContestFile, RunsFile};
use leptos::{logging::*, *};

use crate::{api::create_secret_runs, model::ContestSignal, views::contest::ContestPanel};

#[derive(Debug)]
pub struct State {
    is_started: bool,
    driver: RevelationDriver,
}

impl State {
    fn new(contest: ContestFile, runs: RunsFile, sede: &Sede) -> Self {
        let sub_contest = contest.filter_sede(sede);
        Self {
            is_started: false,
            driver: RevelationDriver::new(sub_contest, runs),
        }
    }

    fn step_forward(&mut self) {
        if self.is_started && self.driver.len() > 0 {
            self.driver.reveal_step();
        } else {
            self.is_started = true;
        }
    }

    fn jump_team_forward(&mut self) {
        if self.is_started {
            self.driver.jump_team_forward();
        } else {
            self.is_started = true
        }
    }

    fn jump_team_back(&mut self) {
        let n = self.driver.len();
        self.reveal_top_n(n + 1)
    }

    fn step_back(&mut self) {
        self.is_started = true;
        self.driver.back_one();
    }

    fn reveal_top_n(&mut self, n: usize) {
        self.is_started = true;
        self.driver.restart();
        self.driver
            .reveal_top_n(n)
            .inspect_err(|err| error!("failed step: {err:?}"))
            .ok();
    }

    fn reveal_all(&mut self) {
        self.driver
            .reveal_top_n(0)
            .inspect_err(|err| error!("failed step: {err:?}"))
            .ok();
        self.is_started = false;
    }

    fn reset(&mut self) {
        self.is_started = false;
        self.driver.restart();
    }
}

#[component]
pub fn RevelationPanel<'cs>(
    state: ReadSignal<State>,
    contest_signal: &'cs ContestSignal,
    sede: Signal<Rc<Sede>>,
) -> impl IntoView {
    let center = Signal::derive(move || {
        state
            .with(|state| state.is_started.then_some(state.driver.peek().cloned()))
            .flatten()
    });
    let contest = Signal::derive(move || state.with(|state| state.driver.contest().clone()));

    view! { <ContestPanel contest_signal contest center sede /> }
}

#[component]
pub fn Control(state: WriteSignal<State>) -> impl IntoView {
    let handle = window_event_listener(ev::keydown, move |ev| match ev.code().as_str() {
        "ArrowLeft" => state.update(|d| d.step_back()),
        "ArrowRight" => state.update(|d| d.step_forward()),
        "ArrowUp" => state.update(|d| d.jump_team_forward()),
        "ArrowDown" => state.update(|d| d.jump_team_back()),
        "Backspace" => state.update(|d| d.reset()),
        code => log!("ev code: {code}"),
    });
    on_cleanup(move || handle.remove());
    view! {
        <div class="commandpanel">
            <button on:click=move |_| { state.update(|d| d.step_back())}>
                {"←"}
            </button>
            <button on:click=move |_| { state.update(|d| d.step_forward())}>
                {"→"}
            </button>
            <button on:click=move |_| { state.update(|d| d.jump_team_forward())}>
                {"↑"}
            </button>
            <button on:click=move |_| { state.update(|d| d.jump_team_back())}>
                {"↓"}
            </button>
            <button on:click=move |_| { state.update(|d| d.reveal_top_n(100))}>
                Top 100
            </button>
            <button on:click=move |_| { state.update(|d| d.reveal_top_n(50))}>
                Top 50
            </button>
            <button on:click=move |_| { state.update(|d| d.reveal_top_n(30))}>
                Top 30
            </button>
            <button on:click=move |_| { state.update(|d| d.reveal_top_n(10))}>
                Top 10
            </button>
            <button on:click=move |_| { state.update(|d| d.reveal_all())}>
                All
            </button>
            <button on:click=move |_| { state.update(|d| d.reset())}>
                Reset
            </button>
        </div>
    }
}

#[component]
pub fn Revelation(sede: Rc<Sede>, runs_file: RunsFile, contest: ContestFile) -> impl IntoView {
    let contest_signal = Rc::new(ContestSignal::new(&contest));
    let contest = contest.clone();
    let driver = State::new(contest, runs_file.clone(), &sede);
    let (get_sede, _) = create_signal(sede.clone());

    let (get_driver, set_driver) = create_signal(driver);

    let effect_contest_signal = contest_signal.clone();
    create_effect(move |_| {
        get_driver.with(|state| {
            let contest = state.driver.contest();
            effect_contest_signal.update(contest.teams.values().map(|t| t.login.as_str()), contest)
        });
    });

    view! {
        <Control state=set_driver />
        <div class="revelationpanel">
            <RevelationPanel contest_signal=&contest_signal state=get_driver sede=get_sede.into() />
        </div>
    }
}

#[component]
pub fn Reveleitor(sede: Rc<Sede>, secret: String, contest: ContestFile) -> impl IntoView {
    let all_runs = create_local_resource(
        || (),
        move |()| {
            let secret = secret.clone();
            create_secret_runs(secret)
        },
    );

    move || {
        let contest = contest.clone();
        all_runs.get().map(|runs_file| {
            view! { <Revelation sede=sede.clone() runs_file contest /> }
        })
    }
}
