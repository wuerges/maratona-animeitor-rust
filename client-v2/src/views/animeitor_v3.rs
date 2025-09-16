use futures::StreamExt;
use leptos::prelude::*;

fn create_timer(contest: String) -> ArcReadSignal<Option<(sdk::Time, sdk::Time)>> {
    let stream = crate::api::create_timer_v3(contest);

    let scanned = stream.scan(sdk::Time::unknown(), |state, next| {
        let previous = *state;
        *state = next;

        std::future::ready(Some((previous, next)))
    });

    ArcReadSignal::from_stream(scanned)
}

#[component]
pub fn Root() -> impl IntoView {
    let timer = create_timer("brasil".to_string());

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

    view! {
        <p> Yay</p>
        {timer_view}
    }
}
