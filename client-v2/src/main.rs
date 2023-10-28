use client_v2::{config::Config, contest::Contest, runs::Runs, timer::Timer};
use leptos::*;

fn main() {
    mount_to_body(|| {
        view! {
            <Timer />
            <Runs />
            <Config />
            <Contest />
        }
    })
}
