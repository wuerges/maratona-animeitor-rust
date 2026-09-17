use animeitor_client::views::{global_settings::provide_global_settings, sedes::Sedes};
use leptos::{mount::mount_to_body, *};

pub fn main() {
    console_log::init_with_level(log::Level::Debug).expect("failed to init console_log");
    console_error_panic_hook::set_once();

    // mount_to_body initializes the executor, but we spawn before mounting
    // to load the runtime config; initialize it here (idempotent).
    #[cfg(target_family = "wasm")]
    let _ = any_spawner::Executor::init_wasm_bindgen();

    leptos::task::spawn_local(async move {
        let config = client_sdk::SdkConfig::load().await;
        animeitor_client::init_config(config);

        mount_to_body(|| {
            provide_global_settings();

            view! {
                <Sedes />
                // <Runs />
                // <Config />
            }
        })
    })
}
