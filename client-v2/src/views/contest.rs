use data::{ContestFile, RunsPanelItem};
use leptos::*;

use crate::{model::provide_contest, views::timer::Timer};

#[component]
pub fn Contest() -> impl IntoView {
    let (contest, panel) = provide_contest();

    move || {
        panel.with(|panel| {
            contest.with(|contest| match contest {
                Some(contest) => everything(contest, panel).into_view(),
                None => view! {<p> Contest is none =/ </p>}.into_view(),
            })
        })
    }
}

fn runs_panel(panel: &Vec<RunsPanelItem>) -> impl IntoView {
    view! {
        <p> Panel!: "`"{format!("{:#?}", panel.iter().take(30).collect::<Vec<_>>())}"'" </p>
    }
}

fn contest_panel(contest: &ContestFile) -> impl IntoView {
    view! {
        <p> Contest!: "`"{format!("{:#?}", contest)}"'" </p>
    }
}

pub fn everything(contest: &ContestFile, panel: &Vec<RunsPanelItem>) -> impl IntoView {
    view! {
        <body style="height: 1px">
            <div style="display: flex; width: 320px;">
                <div style="display: flex; flex-direction: column; width: 320px;">
                    // <Sedes />
                    <Timer />
                    <div class="submission-title"> Últimas Submissões </div>
                    {runs_panel(panel)}
                </div>
                <div class="automatic" style="margin-left: 8px;">
                    {contest_panel(contest)}
                </div>
            </div>
        </body>
    }
}
