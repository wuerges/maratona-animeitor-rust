use leptos::prelude::*;

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

    let is_frozen = move || time.with(|t| t.1 >= score_freeze_time_in_minutes as i64 * 60);

    let hour_class = move || time.with(|(p, c)| format!("hora {}", changed(hor(*p), hor(*c))));
    let mins_class = move || time.with(|(p, c)| format!("minuto {}", changed(min(*p), min(*c))));
    let secs_class = move || time.with(|(p, c)| format!("segundo {}", changed(seg(*p), seg(*c))));

    let hours = move || time.with(|t| f(hor(t.1)));
    let mins = move || time.with(|t| f(min(t.1)));
    let secs = move || time.with(|t| f(seg(t.1)));

    view! {
        <div class="timer" class:frozen=is_frozen>
            <span class=hour_class>{hours} </span>
            <span class="sep"> ":" </span>
            <span class=mins_class>{mins} </span>
            <span class="sep"> ":" </span>
            <span class=secs_class>{secs} </span>
        </div>
    }
}
