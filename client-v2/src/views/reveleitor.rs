use data::{
    configdata::{ConfigContest, Sede},
    revelation::RevelationDriver,
    ContestFile, RunsFile,
};
use leptos::{logging::log, *};

use crate::{
    api::{create_config, create_contest, create_secret_runs},
    views::contest::ContestPanel,
};

#[derive(Debug, Clone, Default)]
struct Steps {
    count: u32,
}

#[component]
pub fn RevelationPanel(driver: ReadSignal<RevelationDriver>, sede: Sede) -> impl IntoView {
    move || {
        driver.with(|driver| {
            let contest = driver.contest().clone();
            let center = driver.peek().cloned();
            log!("center: {center:?}");
            // let center = None;

            view! { <ContestPanel contest center sede=Some(&sede)/> }
        })
    }
}

#[component]
pub fn Control(driver: WriteSignal<RevelationDriver>) -> impl IntoView {
    view! {
        <div class="commandpanel">
            <button on:click=move |_| { driver.update(|d| { d.reveal_step().ok(); })}>
                {"←"}
            </button>
            <button on:click=move |_| { driver.update(|d| { d.reveal_step().ok(); })}>
                {"→"}
            </button>
            <button on:click=move |_| { driver.update(|d| { d.restart(); d.reveal_top_n(100).ok(); })}>
                Top 100
            </button>
            <button on:click=move |_| { driver.update(|d| { d.restart(); d.reveal_top_n(50).ok(); })}>
                Top 50
            </button>
            <button on:click=move |_| { driver.update(|d| { d.restart(); d.reveal_top_n(30).ok(); })}>
                Top 30
            </button>
            <button on:click=move |_| { driver.update(|d| { d.restart(); d.reveal_top_n(10).ok(); })}>
                Top 10
            </button>
            <button on:click=move |_| { driver.update(|d| { d.reveal_top_n(0).ok(); })}>
                All
            </button>
            <button on:click=move |_| { driver.update(|d| { d.restart(); })}>
                Reset
            </button>
            <div>
                Times: {0}
            </div>
        </div>
    }
}

#[component]
pub fn Revelation(
    sede: Sede,
    runs_file: RunsFile,
    contest: ContestFile,
    config: ConfigContest,
) -> impl IntoView {
    let (get_step, set_step) = create_signal(Steps::default());

    move || {
        let sub_contest = contest.clone().filter_sede(&sede);
        let driver = RevelationDriver::new(sub_contest, runs_file.clone()).ok();
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
    let config = create_local_resource(|| (), |()| create_config());

    move || match (all_runs.get(), contest.get(), config.get()) {
        (Some(runs_file), Some(contest), Some(config)) => {
            Some(view! { <Revelation sede=sede.clone() runs_file contest config /> })
        }
        _ => None,
    }
}
