use data::configdata::{ConfigContest, SedeEntry};
use leptos::*;
use leptos_router::*;

#[component]
fn Sede(sede: SedeEntry) -> impl IntoView {
    view! {
        <span class="sedeslink">
            <A href=format!("?sede={}",sede.name)> {sede.name} </A>
        </span>
    }
}

#[component]
pub fn Navigation(config_contest: Resource<(), ConfigContest>) -> impl IntoView {
    move || {
        view! {
            <div class="sedesnavigation">
                <Suspense
                    fallback=||view! {<p> Loading... </p> }
                >
                    {move || {
                        config_contest.get().map(|contest| {
                            contest
                            .sedes.unwrap_or_default()
                            .iter()
                            .cloned()
                            .map(|sede| {
                                view! {<Sede sede />}
                            })
                            .collect_view()
                        })
                    }}
                </Suspense>
            </div>
        }
    }
}
