use client_v2::views::sedes::Sedes;
use leptos::*;

pub fn main() {
    mount_to_body(|| {
        view! {
            <Sedes />
            // <Runs />
            // <Config />
        }
    })
}
