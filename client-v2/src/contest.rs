use leptos::*;

use data::ContestFile;

use crate::request_signal::create_request_signal;

fn create_contest() -> ReadSignal<Option<ContestFile>> {
    let contest_message = create_request_signal("http://localhost:9000/api/contest", None);

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
