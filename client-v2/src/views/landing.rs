//! Landing page: lists the active events (`/` and `/animeitor/`).

use leptos::prelude::*;

use crate::api::create_events;

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
                                .map(|name| {
                                    let link = format!("/animeitor/{name}/");
                                    let href = link.clone();
                                    view! { <li><a href=href>{link}</a></li> }
                                })
                                .collect_view()
                        })
                    }}
                </Suspense>
            </ul>
        </div>
    }
}
