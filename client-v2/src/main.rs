use client_v2::views::contest::Contest;
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
