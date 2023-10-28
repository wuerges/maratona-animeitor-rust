use leptos::*;

use data::configdata::ConfigContest;

use crate::request_signal::create_request_signal;

fn create_config() -> ReadSignal<Option<ConfigContest>> {
    let config_message = create_request_signal("http://localhost:9000/api/config", None);

    config_message
}

#[component]
pub fn Config() -> impl IntoView {
    let config = create_config();

    move || match config.get() {
        Some(config) => view! {
            <p> Config!: "`"{format!("{:#?}", config)}"'" </p>
        },
        None => view! {<p> Config is none =/ </p>},
    }
}
