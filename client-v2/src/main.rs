use client_v2::{config::Config, contest::Contest, timer::Timer};
use leptos::*;

fn main() {
    mount_to_body(|| {
        view! {
            <Timer />
            <Config />
            <Contest />
        }
    })
}
