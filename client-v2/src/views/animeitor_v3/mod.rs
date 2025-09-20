mod timer;

use futures::StreamExt;
use leptos::prelude::*;
use sdk::{components::Data, ContestParameters};

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

#[component]
pub fn Root() -> impl IntoView {
    let timer = create_timer("brasil".to_string());

    let parameters_resource =
        LocalResource::new(|| create_contest_parameters("brasil".to_string()));

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
        let contest = Contest::new(parameters);

        Some(view! {
            <>
                <p> contest was loaded </p>
                <p>  </p>
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
