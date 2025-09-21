use leptos::{leptos_dom::logging::console_log, prelude::*};

fn f(n: i64) -> String {
    format!("{:0>2}", n)
}

fn seg(n: i64) -> i64 {
    n % 60
}
fn min(n: i64) -> i64 {
    (n / 60) % 60
}
fn hor(n: i64) -> i64 {
    n / 60 / 60
}
fn changed(a: i64, b: i64) -> &'static str {
    if a == b {
        "same"
    } else {
        "changed"
    }
}

#[component]
pub fn Timer(
    current_time: Signal<Option<sdk::Time>>,
    score_freeze_time_in_minutes: u32,
) -> impl IntoView {
    let time = RwSignal::new((
        sdk::Time::unknown().time_in_seconds - 2,
        sdk::Time::unknown().time_in_seconds - 1,
    ));

    Effect::new(move || {
        let new_time = current_time
            .get()
            .unwrap_or(sdk::Time::unknown())
            .time_in_seconds;

        time.update(|previous| {
            previous.0 = previous.1;
            previous.1 = new_time;
        });
    });

    move || {
        time.with(|(previous, current)| {
            let is_frozen = *current >= score_freeze_time_in_minutes as i64 * 60;

            let (hours, mins, secs) = (hor(*current), min(*current), seg(*current));

            let (previous_hours, previous_mins, previous_secs) =
                (hor(*previous), min(*previous), seg(*previous));

            let hour_class = format!("hora {}", changed(hours, previous_hours));
            let mins_class = format!("minuto {}", changed(mins, previous_mins));
            let secs_class = format!("segundo {}", changed(secs, previous_secs));

            view! {
                <div class="timer" class:frozen=is_frozen>
                    <span class=hour_class>{hours} </span>
                    <span class="sep"> ":" </span>
                    <span class=mins_class>{mins} </span>
                    <span class="sep"> ":" </span>
                    <span class=secs_class>{secs} </span>

                </div>
            }
        })
    }
}
