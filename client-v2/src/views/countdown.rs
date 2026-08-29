//! Countdown screen shown while the contest has not started.
//!
//! The public API serves nothing about the contest before the start, so the
//! event/contest names come from the URL path.

use data::TimerData;
use leptos::prelude::*;

use crate::api::EventContest;

fn pad(n: u64) -> String {
    format!("{n:0>2}")
}

#[component]
pub fn Countdown(ec: EventContest, timer: ReadSignal<(TimerData, TimerData)>) -> impl IntoView {
    // The names never change; only the remaining time is reactive.
    let event_name = ec.event;
    let contest_name = ec.contest;

    view! {
        <div class="countdown">
            <div class="event-name">{event_name}</div>
            <div class="contest-name">{contest_name}</div>
            <div class="remaining">
                "Faltam "
                {move || {
                    let (time_data, _) = timer.get();
                    let remaining = time_data.current_time.unsigned_abs();
                    view! {
                        <span class="horas">{pad(remaining / 3600)}</span>":"
                        <span class="minutos">{pad((remaining / 60) % 60)}</span>":"
                        <span class="segundos">{pad(remaining % 60)}</span>
                    }
                }}
                " para o início"
            </div>
        </div>
    }
}
