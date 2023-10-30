use leptos::*;

use crate::model::provide_contest;

#[component]
pub fn Contest() -> impl IntoView {
    let (contest, panel) = provide_contest();

    move || {
        panel.with(|panel| {
            contest.with(|contest| match contest {
                Some(contest) => view! {
                    <p> Panel!: "`"{format!("{:#?}", panel.iter().take(30).collect::<Vec<_>>())}"'" </p>
                    <p> Contest!: "`"{format!("{:#?}", contest)}"'" </p>
                }
                .into_view(),
                None => view! {<p> Contest is none =/ </p>}.into_view(),
            })
        })
    }
}
