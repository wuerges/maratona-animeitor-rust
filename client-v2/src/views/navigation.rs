use data::configdata::{ConfigContest, SedeEntry};
use leptos::*;
use leptos_router::*;

#[component]
fn Sede(sede: SedeEntry) -> impl IntoView {
    view! {
        <span class="sedeslink">
            <A href=format!("/sedes/{}", sede.name)> {sede.name} </A>
        </span>
    }
}

#[component]
pub fn Navigation(
    config_contest: Resource<(), ConfigContest>,
    contest_name: ReadSignal<Option<String>>,
) -> impl IntoView {
    move || {
        config_contest.with(|contest| match contest {
            Some(contest) => {
                let contest_name = contest_name.get();
                let sedes = contest
                    .sedes
                    .iter()
                    .filter(|sede| contest_name.is_none() || contest_name == sede.contest);
                view! {
                    <div class="sedesnavigation">
                        {sedes.cloned().map(|sede| {
                            view!{<Sede sede />}
                        }).collect_view()}
                    </div>
                }
            }
            None => {
                view! {<div> No contest selected </div>}
            }
        })
    }
}
