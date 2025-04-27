use std::{collections::HashMap, hash::Hash};

use leptos::prelude::*;

pub trait Compress {
    type Key: Eq + Hash + Clone + Send + Sync + 'static;
    fn key(&self) -> Signal<Option<Self::Key>>;
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
    placements: Signal<Vec<T::Key>>,
    center: Signal<Option<T::Key>>,
) -> impl IntoView
where
    T: Compress + 'static,
    T::Key: Send + Sync,
{
    let placements = Memo::new(move |_| {
        placements.with(|p| {
            p.iter()
                .enumerate()
                .map(|(i, login)| (login.clone(), i + 1))
                .collect::<HashMap<T::Key, usize>>()
        })
    });

    let p_center = Signal::derive(move || {
        placements.with(|placements| {
            center.with(|center| {
                let center = center.as_ref()?;
                let position = placements.get(center)?;
                Some(*position)
            })
        })
    });

    inner_compress_placements(children, placements.into(), p_center)
}

fn inner_compress_placements<T>(
    children: Vec<T>,
    placements: Signal<HashMap<T::Key, usize>>,
    center: Signal<Option<usize>>,
) -> impl IntoView
where
    T: Compress + 'static,
{
    children
        .into_iter()
        .map(|c| {
            let key = c.key();
            let signal = Memo::new(move |_| {
                let key = key.get()?;
                placements.with(move |p| p.get(&key).copied())
            });
            c.view_in_position(signal.into(), center)
        })
        .collect_view()
}
