mod timer;

use std::sync::Arc;

use async_lock::RwLock;
use futures::{Stream, StreamExt};
use leptos::{leptos_dom::logging::console_log, prelude::*, task::spawn};
use sdk::{components::Data, ContestParameters, Run, SiteConfiguration};

use crate::{
    api::url_prefix,
    model::animeitor_v3::contest::Contest,
    net::{request_signal::create_request, websocket_stream::create_websocket_stream_2},
    views::animeitor_v3::timer::Timer,
};

fn create_runs(contest: String) -> impl Stream<Item = Run> {
    let prefix = url_prefix();
    create_websocket_stream_2(&format!("{prefix}/contests/{contest}/runs-websocket"))
}

fn create_updater(
    runs_stream: impl Stream<Item = Run> + Send + 'static,
    contest: Arc<RwLock<Contest>>,
    sites: Arc<SiteConfiguration>,
) {
    spawn(async move {
        console_log("updater started");
        runs_stream
            .ready_chunks(1000)
            .for_each(async |runs| {
                let mut lock = contest.write().await;
                for run in runs {
                    lock.judge_run(&run);
                }
                console_log(&format!("sites {:?}", sites));
            })
            .await;
        console_log("updater finished");
    });
}

async fn create_contest_parameters(contest: String) -> Data<ContestParameters> {
    let prefix = url_prefix();
    create_request(&format!("{prefix}/contests/{contest}/parameters")).await
}

async fn create_site_configuration(contest: String) -> Data<SiteConfiguration> {
    let prefix = url_prefix();
    create_request(&format!("{prefix}/contests/{contest}/sites")).await
}

fn create_timer(contest: String) -> ArcReadSignal<Option<sdk::Time>> {
    let stream = crate::api::create_timer_v3(contest);

    ArcReadSignal::from_stream(stream)
}

#[component]
pub fn Root() -> impl IntoView {
    let parameters_resource =
        LocalResource::new(|| create_contest_parameters("brasil".to_string()));

    let site_resource = LocalResource::new(|| create_site_configuration("brasil".to_string()));

    let parameters = move || {
        let parameters = parameters_resource.get()?.data;
        let score_freeze_time_in_minutes = parameters.score_freeze_time_in_minutes;
        let sites = Arc::new(site_resource.get()?.data);
        let contest = Arc::new(RwLock::new(Contest::new(parameters)));

        let current_time = create_timer("brasil".to_string());
        let runs_stream = create_runs("brasil".to_string());

        let sites_view = sites
            .sites
            .iter()
            .map(|s| {
                view! {<p> Site {s.name.to_string()} </p>}
            })
            .collect_view();

        create_updater(runs_stream, contest, sites.clone());
        // let teams = contest.
        //     .teams()
        //     .map(|t| {
        //         view! {
        //             <p> Team: {t.info().name.to_string()}</p>
        //         }
        //     })
        //     .collect_view();

        Some(view! {
            <>
                <Timer current_time=current_time.into() score_freeze_time_in_minutes />
                <p> contest was loaded </p>
                <h1> Teams </h1>

                <h1> Sites </h1>

                {sites_view}
            </>
        })
    };

    view! {
        <p> Yay</p>


        <Suspense>
            {parameters}
        </Suspense>
    }
}
