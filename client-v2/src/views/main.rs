use leptos::*;

use crate::views::{config::Config, contest::Contest, runs::Runs, timer::Timer};

pub fn main() {
    mount_to_body(|| {
        view! {
            <Timer />
            <Runs />
            <Config />
            <Contest />
        }
    })
}
