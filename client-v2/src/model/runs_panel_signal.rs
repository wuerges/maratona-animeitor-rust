use std::{collections::BTreeMap, sync::Arc};

use data::{configdata::Sede, RunsPanelItem};
use itertools::Itertools;
use leptos::prelude::*;

#[derive(Clone)]
pub struct RunPanelItemSignal {
    pub panel_item: RwSignal<Option<RunsPanelItem>>,
}

impl RunPanelItemSignal {
    fn new() -> Self {
        Self {
            panel_item: RwSignal::new(None),
        }
    }
    fn set(&self, new_item: RunsPanelItem) {
        self.panel_item.set(Some(new_item));
    }
}

pub struct RunsPanelItemManager {
    pub items: Vec<RunPanelItemSignal>,
    untracked: RwSignal<Untracked>,
}

#[derive(Debug, Default)]
struct Untracked {
    index: BTreeMap<i64, usize>,
    rot: usize,
}

impl RunsPanelItemManager {
    pub const MAX: usize = 299;
    pub fn new() -> Self {
        Self {
            items: (0..=Self::MAX)
                .map(|_| RunPanelItemSignal::new())
                .collect_vec(),
            untracked: Default::default(),
        }
    }

    pub fn push(&self, new_item: RunsPanelItem) {
        let found = self
            .untracked
            .with_untracked(|idx| idx.index.get(&new_item.id).copied());

        match found {
            Some(found) => self.items[found].set(new_item),
            None => {
                self.untracked.update_untracked(|u| {
                    u.index.insert(new_item.id, u.rot);
                    self.items[u.rot].set(new_item);
                    u.rot += 1;
                    u.rot %= Self::MAX;
                });
            }
        }
    }

    pub fn placements_for_sede(&self, sede: Signal<Arc<Sede>>) -> Signal<Vec<u64>> {
        let signals = self.items.clone();
        Memo::new(move |_| {
            sede.with(|s| {
                signals
                    .iter()
                    .filter_map(move |item| {
                        let (login, order) = item
                            .panel_item
                            .with(|v| v.as_ref().map(|i| (i.team_login.clone(), i.order)))?;
                        s.team_belongs_str(&login).then_some(order)
                    })
                    .sorted()
                    .rev()
                    .collect_vec()
            })
        })
        .into()
    }
}

impl Default for RunsPanelItemManager {
    fn default() -> Self {
        Self::new()
    }
}
