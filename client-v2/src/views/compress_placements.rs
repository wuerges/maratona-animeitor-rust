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

/// Takes a vector o children components
/// each component must be identifiable by a Key
/// and should be able to build its view from a position and center signals
///
/// - `children`: The array of components to compress.
/// - `placements`: A signal with an array of string, that identify each component.
///                 Only components in the `placements` array will be shown.
/// - `center`: Which child should be the center component?
pub fn compress_placements<T>(
    children: Vec<T>,
    placements: Signal<Vec<String>>,
    center: Signal<Option<String>>,
) -> impl IntoView
where
    T: Compress + 'static,
{
    let placements = create_memo(move |_| {
        placements.with(|p| {
            p.iter()
                .enumerate()
                .map(|(i, login)| (login.clone(), i + 1))
                .collect::<HashMap<String, usize>>()
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
