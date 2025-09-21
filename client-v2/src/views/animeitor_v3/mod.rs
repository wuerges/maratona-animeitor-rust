mod timer;

use futures::StreamExt;
use leptos::prelude::*;
use sdk::{components::Data, ContestParameters, SiteConfiguration};

use crate::{
    api::url_prefix, model::animeitor_v3::contest::Contest, net::request_signal::create_request,
};

fn create_timer(contest: String) -> ArcReadSignal<Option<(sdk::Time, sdk::Time)>> {
    let stream = crate::api::create_timer_v3(contest);

    let scanned = stream.scan(sdk::Time::unknown(), |state, next| {
        let previous = *state;
        *state = next;

        std::future::ready(Some((previous, next)))
    });

    ArcReadSignal::from_stream(scanned)
}

async fn create_contest_parameters(contest: String) -> Data<ContestParameters> {
    let prefix = url_prefix();
    create_request(&format!("{prefix}/contests/{contest}/parameters")).await
}

async fn create_site_configuration(contest: String) -> Data<SiteConfiguration> {
    let prefix = url_prefix();
    create_request(&format!("{prefix}/contests/{contest}/sites")).await
}

#[component]
pub fn Root() -> impl IntoView {
    let timer = create_timer("brasil".to_string());

    let parameters_resource =
        LocalResource::new(|| create_contest_parameters("brasil".to_string()));

    let site_resource = LocalResource::new(|| create_site_configuration("brasil".to_string()));

    let timer_view = move || {
        timer.get().map(
            |(
                sdk::Time {
                    time_in_seconds: prev,
                },
                sdk::Time {
                    time_in_seconds: next,
                },
            )| {
                view! { <p> Time: {prev}/{next} </p> }
            },
        )
    };

    let parameters = move || {
        let parameters = parameters_resource.get()?.data;
        let sites = site_resource.get()?.data;
        let contest = Contest::new(parameters);

        let teams = contest
            .teams()
            .map(|t| {
                view! {
                    <p> Team: {t.info().name.to_string()}</p>
                }
            })
            .collect_view();

        let sites_view = sites
            .sites
            .iter()
            .map(|s| {
                view! {<p> Site {s.name.to_string()} </p>}
            })
            .collect_view();

        Some(view! {
            <>
                <p> contest was loaded </p>
                <h1> Teams </h1>
                {teams}
                <h1> Sites </h1>
                {sites_view}
            </>
        })
    };

    view! {
        <p> Yay</p>
        {timer_view}

        <Suspense>
            {parameters}
        </Suspense>
    }
}
