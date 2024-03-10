use data::configdata::ConfigContest;
use leptos::{component, view, CollectView, IntoView, ReadSignal, Signal, SignalGet, SignalWith};

#[component]
pub fn Navigation(
    config_contest: Signal<Option<ConfigContest>>,
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
                        {sedes.map(|sede| {
                            view! {
                                <span class="sedeslink">
                                    <a href="/sede"> {&sede.name} </a>
                                </span>
                            }
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
