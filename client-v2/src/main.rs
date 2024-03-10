use client_v2::views::countdown::Countdown;
use leptos::*;

pub fn main() {
    mount_to_body(|| {
        view! {
            <Countdown />
            // <Runs />
            // <Config />
        }
    })
}
