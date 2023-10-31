use client_v2::views::{config::Config, contest::Contest, runs::Runs, timer::Timer};
use leptos::*;

pub fn main() {
    mount_to_body(|| {
        view! {
            // <Timer />
            <Contest />
            // <Runs />
            // <Config />
        }
    })
}
