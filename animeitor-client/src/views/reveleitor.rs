use std::{collections::HashMap, sync::Arc, sync::Mutex};

use data::{configdata::Sede, ContestFile, RunsFile};
use leptos::{ev, logging::*, prelude::*};

use client_model::contest_signal::ContestSignal;
use client_model::RevelationDriver;

use crate::{api::create_secret_runs, views::contest::ContestPanel};

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
        if self.is_started && !self.driver.is_empty() {
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
pub fn RevelationPanel(
    original_contest: Arc<ContestFile>,
    state: ReadSignal<State>,
    contest_signal: Arc<ContestSignal>,
    sede: Signal<Arc<Sede>>,
) -> impl IntoView {
    let center = Signal::derive(move || {
        state
            .with(|state| state.is_started.then_some(state.driver.peek().cloned()))
            .flatten()
    });

    view! { <ContestPanel original_contest contest_signal center titulo=None.into() sede /> }
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
fn RevelationWelcome(on_confirm: Callback<()>) -> impl IntoView {
    let dialog = NodeRef::<leptos::html::Dialog>::new();
    Effect::new(move |_| {
        if let Some(dialog) = dialog.get() {
            let _ = dialog.show_modal();
        }
    });
    view! {
        <dialog
            node_ref=dialog
            class="revelation-welcome"
            aria-labelledby="revelation-welcome-title"
            aria-describedby="revelation-welcome-description"
            on:cancel=move |event: web_sys::Event| event.prevent_default()
        >
            <h1 id="revelation-welcome-title">"These are not the final scores"</h1>
            <p id="revelation-welcome-description">
                "Reveleitor starts with unrevealed submissions. Reveal all submissions to reach the final scores and standings."
            </p>
            <h2>"Keyboard controls"</h2>
            <dl>
                <dt><kbd>"→"</kbd></dt><dd>"Reveal the next submission"</dd>
                <dt><kbd>"←"</kbd></dt><dd>"Go back one submission"</dd>
                <dt><kbd>"↑"</kbd></dt><dd>"Step up one team"</dd>
                <dt><kbd>"↓"</kbd></dt><dd>"Step down one team"</dd>
                <dt><kbd>"Backspace"</kbd></dt><dd>"Reset the revelation"</dd>
                <dt><kbd>"Y"</kbd></dt><dd>"Show or hide the team photo"</dd>
                <dt><kbd>"M"</kbd></dt><dd>"Toggle automatic team-song playback"</dd>
            </dl>
            <p>"Photos and songs are available when configured for this contest."</p>
            <button autofocus on:click=move |_| on_confirm.run(())>"OK"</button>
        </dialog>
    }
}

#[component]
pub fn Revelation(sede: Arc<Sede>, runs_file: RunsFile, contest: ContestFile) -> impl IntoView {
    log!("revelation");
    let contest_signal = Arc::new(ContestSignal::new(&contest));
    let contest = contest.clone();
    let original_contest = Arc::new(contest.clone());
    let driver = State::new(contest, runs_file, &sede);
    let (get_sede, _) = signal(sede.clone());

    let (get_driver, set_driver) = signal(driver);

    let effect_contest_signal = contest_signal.clone();

    let team_ids = Arc::new(Mutex::new(HashMap::new()));

    Effect::new(move |_| {
        get_driver.with(|state| {
            let contest = state.driver.contest();
            let mut id_map = team_ids.lock().unwrap();

            let mut changed_logins = vec![];
            for team in contest.teams.values() {
                let id_changed = id_map.get(&team.login).is_none_or(|id| &team.id != id);

                if id_changed {
                    changed_logins.push(team.login.as_str());
                    id_map.insert(team.login.clone(), team.id);
                }
            }

            effect_contest_signal.update(changed_logins.into_iter(), contest)
        });
    });

    let (acknowledged, set_acknowledged) = signal(false);
    view! {
        <Show
            when=move || acknowledged.get()
            fallback=move || view! {
                <RevelationWelcome on_confirm=Callback::new(move |_| set_acknowledged.set(true)) />
            }
        >
            <Control state=set_driver />
            <div class="revelationpanel">
                <RevelationPanel original_contest=original_contest.clone() contest_signal=contest_signal.clone() state=get_driver sede=get_sede.into() />
            </div>
        </Show>
    }
}

#[component]
pub fn Reveleitor(
    sede: Arc<Sede>,
    secret: String,
    contest: Arc<ContestFile>,
    event_contest: crate::api::EventContest,
    export_generation: u64,
) -> impl IntoView {
    log!("reveleitor");
    let export = expect_context::<crate::offline::OfflineExportContext>();
    let all_runs = LocalResource::new(move || {
        log!("fetching secret runs");
        let secret = secret.clone();
        let event_contest = event_contest.clone();
        create_secret_runs(secret, event_contest)
    });

    let contest = ContestFile::clone(&contest);
    Suspend::new(async move {
        let runs_file = all_runs.await;
        export.publish(
            export_generation,
            crate::offline::OfflineExportInputs {
                contest: contest.clone(),
                runs: runs_file.clone(),
                sede: sede.entry.clone(),
            },
        );

        view! {
            <Revelation sede=sede.clone() runs_file contest />
        }
    })
}
