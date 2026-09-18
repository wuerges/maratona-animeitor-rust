//! Countdown screen shown while the contest has not started.
//!
//! The public API serves nothing about the contest before the start, so the
//! event/contest names come from the URL path.

use data::TimerData;
use leptos::prelude::*;

use super::timer::Timer;
use crate::api::EventContest;

#[component]
pub fn Countdown(ec: EventContest, timer: ReadSignal<(TimerData, TimerData)>) -> impl IntoView {
    // The names never change; only the remaining time is reactive.
    let event_name = ec.event;
    let contest_name = ec.contest;

    view! {
        <div class="countdown">
            <Timer timer />
            <div class="event-name">{event_name}</div>
            <div class="contest-name">{contest_name}</div>
            <p class="countdown-message">"Aguardando o início da competição"</p>
        </div>
    }
}
