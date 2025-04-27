use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::Arc,
};

use data::{configdata::Sede, RunsPanelItem};
use itertools::Itertools;
use leptos::prelude::*;

#[derive(Clone)]
pub struct RunPanelItemSignal {
    pub panel_item: RwSignal<RunsPanelItem>,
    pub team_login: String,
}

impl RunPanelItemSignal {
    fn new(panel_item: RunsPanelItem) -> Self {
        Self {
            team_login: panel_item.team_login.clone(),
            panel_item: RwSignal::new(panel_item),
        }
    }
    fn set(&self, new_item: RunsPanelItem) {
        self.panel_item.set(new_item);
    }
}

pub struct RunsPanelItemManager {
    pub items: RwSignal<BTreeMap<i64, RunPanelItemSignal>>,
}

impl RunsPanelItemManager {
    pub const MAX: usize = 29;
    pub fn new() -> Self {
        Self {
            items: Default::default(),
        }
    }

    pub fn push(&self, new_item: RunsPanelItem) {
        self.items.update(|items| {
            if items.len() > Self::MAX && !items.contains_key(&new_item.id) {
                items.pop_first();
            }
            match items.entry(new_item.id) {
                Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(RunPanelItemSignal::new(new_item));
                }
                Entry::Occupied(occupied_entry) => occupied_entry.get().set(new_item),
            }
        });
    }

    pub fn placements_for_sede(&self, sede: Signal<Arc<Sede>>) -> Signal<Vec<i64>> {
        let items = self.items.clone();
        Memo::new(move |_| {
            sede.with(|s| {
                items.with(|tree| {
                    tree.iter()
                        .rev()
                        .take(Self::MAX)
                        .filter_map(move |(key, item)| {
                            s.team_belongs_str(&item.team_login).then_some(*key)
                        })
                        .collect_vec()
                })
            })
        })
        .into()
    }
}
