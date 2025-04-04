use std::collections::HashMap;

use leptos::{create_memo, CollectView, IntoView, Signal, SignalWith};

pub trait Compress {
    fn key(&self) -> String;
    fn view_in_position(
        self,
        position: Signal<Option<usize>>,
        center: Signal<Option<usize>>,
    ) -> impl IntoView;
}

pub fn compress_placements<T>(
    children: Vec<T>,
    placements: Signal<HashMap<String, usize>>,
    center: Signal<Option<String>>,
) -> impl IntoView
where
    T: Compress + 'static,
{
    let p_center = move || {
        placements.with(|placements| {
            center.with(|center| {
                let center = center.as_ref()?;
                let position = placements.get(center)?;
                Some(*position)
            })
        })
    };

    inner_compress_placements(children, placements.into(), p_center.into())
}

fn inner_compress_placements<T>(
    children: Vec<T>,
    placements: Signal<HashMap<String, usize>>,
    center: Signal<Option<usize>>,
) -> impl IntoView
where
    T: Compress + 'static,
{
    children
        .into_iter()
        .map(|c| (c.key(), c))
        .map(|(key, c)| {
            let signal = create_memo(move |_| placements.with(|p| p.get(&key).copied()));
            c.view_in_position(signal.into(), center)
        })
        .collect_view()
}
