use leptos::*;

use crate::api::create_contest;

#[component]
pub fn Contest() -> impl IntoView {
    let contest = create_resource(|| (), |_| create_contest());

    move || match contest.get() {
        Some(contest) => view! {
            <p> Contest!: "`"{format!("{:#?}", contest)}"'" </p>
        },
        None => view! {<p> Contest is none =/ </p>},
    }
}
