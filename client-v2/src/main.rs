use client_v2::views::{global_settings::provide_global_settings, sedes::Sedes};
use leptos::{mount::mount_to_body, *};

pub fn main() {
    console_log::init_with_level(log::Level::Debug).expect("failed to init console_log");

    leptos::task::spawn_local(async move {
        let config = client_sdk::SdkConfig::load().await;
        client_v2::init_config(config);

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
