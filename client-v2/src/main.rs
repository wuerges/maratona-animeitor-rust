use client_v2::{contest::Contest, timer::Timer};
use leptos::*;

fn main() {
    mount_to_body(|| {
        view! {
            <Timer />
            <Contest />
        }
    })
}
