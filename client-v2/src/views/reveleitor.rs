use data::{
    configdata::{ConfigContest, Sede},
    revelation::RevelationDriver,
    ContestError, ContestFile, RunsFile,
};
use leptos::{logging::*, *};

use crate::{
    api::{create_config, create_contest, create_secret_runs},
    views::contest::ContestPanel,
};

#[derive(Debug)]
pub struct State {
    started: bool,
    driver: RevelationDriver,
}

impl State {
    fn new(contest: ContestFile, runs: RunsFile) -> Option<Self> {
        Some(Self {
            started: false,
            driver: RevelationDriver::new(contest, runs)
                .inspect_err(|err| error!("failed creating revelation: {err:?}"))
                .ok()?,
        })
    }

    fn step_forward(&mut self) {
        self.driver
            .reveal_step()
            .inspect_err(|err| error!("failed step: {err:?}"))
            .ok();
    }

    fn step_back(&mut self) {
        self.driver
            .back_one()
            .inspect_err(|err| error!("failed step: {err:?}"))
            .ok();
    }

    fn reveal_top_n(&mut self, n: usize) {
        self.driver.restart();
        self.driver
            .reveal_top_n(n)
            .inspect_err(|err| error!("failed step: {err:?}"))
            .ok();
    }

    fn reset(&mut self) {
        self.driver.restart();
    }
}

#[component]
pub fn RevelationPanel(driver: ReadSignal<State>, sede: Sede) -> impl IntoView {
    move || {
        driver.with(|driver| {
            let contest = driver.driver.contest().clone();
            let center = driver.driver.peek().cloned();

            view! { <ContestPanel contest center sede=Some(&sede)/> }
        })
    }
}

#[component]
pub fn Control(driver: WriteSignal<State>) -> impl IntoView {
    view! {
        <div class="commandpanel">
            <button on:click=move |_| { driver.update(|d| d.step_back())}>
                {"←"}
            </button>
            <button on:click=move |_| { driver.update(|d| d.step_forward())}>
                {"→"}
            </button>
            <button on:click=move |_| { driver.update(|d| d.reveal_top_n(100))}>
                Top 100
            </button>
            <button on:click=move |_| { driver.update(|d| d.reveal_top_n(50))}>
                Top 50
            </button>
            <button on:click=move |_| { driver.update(|d| d.reveal_top_n(30))}>
                Top 30
            </button>
            <button on:click=move |_| { driver.update(|d| d.reveal_top_n(10))}>
                Top 10
            </button>
            <button on:click=move |_| { driver.update(|d| d.reveal_top_n(0))}>
                All
            </button>
            <button on:click=move |_| { driver.update(|d| d.reset())}>
                Reset
            </button>
            <div>
                Times: {0}
            </div>
        </div>
    }
}

#[component]
pub fn Revelation(sede: Sede, runs_file: RunsFile, contest: ContestFile) -> impl IntoView {
    move || {
        let sub_contest = contest.clone().filter_sede(&sede);
        let driver = State::new(sub_contest, runs_file.clone());
        driver.map(|driver| {
            let (get_driver, set_driver) = create_signal(driver);

            view! {
                <Control driver=set_driver />
                <RevelationPanel driver=get_driver sede=sede.clone() />
            }
        })
    }
}

#[component]
pub fn Reveleitor(sede: Sede) -> impl IntoView {
    let all_runs = create_local_resource(|| (), |()| create_secret_runs("saltsecret"));
    let contest = create_local_resource(|| (), |()| create_contest());

    move || match (all_runs.get(), contest.get()) {
        (Some(runs_file), Some(contest)) => {
            Some(view! { <Revelation sede=sede.clone() runs_file contest /> })
        }
        _ => None,
    }
}
