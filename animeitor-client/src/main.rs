#[cfg(target_family = "wasm")]
use animeitor_client::views::{global_settings::provide_global_settings, sedes::Sedes};
#[cfg(target_family = "wasm")]
use leptos::{mount::mount_to_body, *};

#[cfg(not(target_family = "wasm"))]
pub fn main() {
    eprintln!(
        "animeitor-client runs in a browser as WebAssembly.\n\
         From the repository root, run `make run-debug-client`, then open http://localhost:8080/."
    );
    std::process::exit(1);
}

#[cfg(target_family = "wasm")]
pub fn main() {
    console_log::init_with_level(log::Level::Debug).expect("failed to init console_log");
    console_error_panic_hook::set_once();

    // mount_to_body initializes the executor, but we spawn before mounting
    // to load the runtime config; initialize it here (idempotent).
    let _ = any_spawner::Executor::init_wasm_bindgen();

    leptos::task::spawn_local(async move {
        if animeitor_client::offline::mount_if_present() {
            return;
        }
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
