mod timer;

use futures::StreamExt;
use leptos::prelude::*;
use sdk::{components::Data, ContestParameters};

use crate::{api::url_prefix, net::request_signal::create_request};

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

    let contest_parameters = LocalResource::new(|| create_contest_parameters("brasil".to_string()));

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
        contest_parameters.with(|p| {
            p.as_ref().map(|data| {
                let ContestParameters {
                    teams,
                    maximum_time_in_minutes,
                    score_freeze_time_in_minutes,
                    penalty_per_wrong_answer,
                    problem_letters,
                } = &data.data;
                view! {
                    <>
                        <p> parameters </p>
                        <p>{*maximum_time_in_minutes}</p>
                        <p>{*score_freeze_time_in_minutes}</p>
                        <p>{*penalty_per_wrong_answer}</p>
                    </>
                }
            })
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
