use std::sync::Arc;

use data::configdata::{Color, Sede};
use leptos::prelude::*;

pub fn get_color(n: usize, sede: &Sede) -> Option<Color> {
    sede.premio(n)
}

fn get_class(color: Color) -> &'static str {
    match color {
        Color::Red => "vermelho",
        Color::Gold => "ouro",
        Color::Silver => "prata",
        Color::Bronze => "bronze",
        Color::Green => "verde",
        Color::Yellow => "amarelo",
    }
}

#[component]
pub fn Placement(placement: usize, sede: Signal<Arc<Sede>>) -> impl IntoView {
    let background_color = Signal::derive(move || {
        sede.with(|sede| {
            get_color(placement, sede)
                .map(get_class)
                .unwrap_or_default()
        })
    });

    view! {
        <div
        // style:background-color=background_color
        class=move || format!("cell quadrado colocacao {}", background_color.get())
        >
        {placement}
        </div>
    }
}
