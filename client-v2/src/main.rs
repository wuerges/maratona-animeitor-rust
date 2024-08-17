use client_v2::views::{global_settings::provide_global_settings, sedes::Sedes};
use leptos::*;

pub fn main() {
    mount_to_body(|| {
        provide_global_settings();

        view! {
            <Sedes />
            // <Runs />
            // <Config />
        }
    })
}
