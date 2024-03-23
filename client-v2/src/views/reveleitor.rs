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

            view! { <RevelationPanel driver=get_driver sede=sede.clone() /> }
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
