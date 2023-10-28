use leptos::*;

use data::ContestFile;

use crate::ws_component::create_websocket_signal;

fn create_contest() -> ReadSignal<Option<ContestFile>> {
    let contest_message = create_websocket_signal("ws://localhost:9000/api/contest", None);

    contest_message
}

#[component]
pub fn Contest() -> impl IntoView {
    let contest = create_contest();

    move || match contest.get() {
        Some(contest) => view! {
            <p> Contest!: "`"{format!("{:#?}", contest)}"'" </p>
        },
        None => view! {<p> Contest is none =/ </p>},
    }
}
