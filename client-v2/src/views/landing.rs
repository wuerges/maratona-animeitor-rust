//! Landing page: lists the active events and their contests (`/` and
//! `/animeitor/`). Before a contest starts its name is not served, so the
//! contest list is empty until then.

use leptos::prelude::*;

use crate::api::{create_contests, create_events};

#[component]
pub fn Landing() -> impl IntoView {
    let events = LocalResource::new(create_events);

    view! {
        <div class="landing">
            <h1>Eventos</h1>
            <ul>
                <Suspense fallback=move || "Carregando...">
                    {move || {
                        events.get().map(|names| {
                            names
                                .into_iter()
                                .map(|event| {
                                    let event_name = event.clone();
                                    let contests = LocalResource::new({
                                        let event = event.clone();
                                        move || create_contests(event.clone())
                                    });
                                    let event_for_links = event;
                                    view! {
                                        <li>
                                            <span class="event-name">{event_name}</span>
                                            <ul>
                                                <Suspense fallback=move || view! { <></> }.into_view()>
                                                    {move || {
                                                        contests.get().map(|names| {
                                                            let event = event_for_links.clone();
                                                            names
                                                                .into_iter()
                                                                .map(move |contest| {
                                                                    let link =
                                                                        format!("/animeitor/{event}/{contest}/");
                                                                    let href = link.clone();
                                                                    view! { <li><a href=href>{link}</a></li> }
                                                                })
                                                                .collect_view()
                                                        })
                                                    }}
                                                </Suspense>
                                            </ul>
                                        </li>
                                    }
                                })
                                .collect_view()
                        })
                    }}
                </Suspense>
            </ul>
        </div>
    }
}
