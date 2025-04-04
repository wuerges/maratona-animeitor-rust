use std::collections::HashMap;

use itertools::Itertools;
use leptos::{create_memo, CollectView, IntoView, Signal, SignalGet, SignalWith};

pub trait Enabler {
    fn is_enabled(&self, t: &str) -> bool;
}
pub trait Compress {
    fn key(&self) -> String;
    fn position(&self) -> Signal<usize>;
    fn view_in_position(
        self,
        position: Signal<Option<usize>>,
        center: Signal<Option<usize>>,
    ) -> impl IntoView;
}

pub fn compress_placements<T, E>(
    children: Vec<T>,
    enabler: Signal<E>,
    center: Signal<Option<String>>,
) -> impl IntoView
where
    T: Compress + 'static,
    E: Enabler,
{
    let signals = children
        .iter()
        .map(|c| (c.key(), c.position()))
        .collect_vec();

    let placements = create_memo(move |_: Option<&HashMap<String, usize>>| {
        enabler.with(|e| {
            signals
                .iter()
                .filter(|(key, _position)| e.is_enabled(key))
                .sorted_by_cached_key(|(_key, position)| position.get())
                .enumerate()
                .map(|(i, (key, _position))| (key.clone(), i + 1))
                .collect()
        })
    });

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
