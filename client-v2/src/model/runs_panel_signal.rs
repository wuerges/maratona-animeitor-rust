use std::{collections::HashSet, sync::Arc};

use data::{configdata::Sede, RunsPanelItem};
use itertools::Itertools;
use leptos::prelude::*;

#[derive(Clone)]
pub struct RunPanelItemSignal {
    pub panel_item: RwSignal<Option<RunsPanelItem>>,
    pub position: RwSignal<usize>,
}

pub struct RunsPanelItemManager {
    pub items: Vec<RunPanelItemSignal>,
    pub push_ids: RwSignal<HashSet<i64>>,
}

impl RunsPanelItemManager {
    pub const MAX: usize = 29;
    pub fn new() -> Self {
        Self {
            items: (0..=Self::MAX)
                .map(|i| RunPanelItemSignal {
                    panel_item: RwSignal::new(None),
                    position: RwSignal::new(i),
                })
                .collect_vec(),
            push_ids: RwSignal::new(HashSet::new()),
        }
    }

    fn append(&self, new_item: RunsPanelItem) {
        for p in &self.items {
            p.position.update(|i| {
                if *i == Self::MAX {
                    *i = 0;
                } else {
                    *i = *i + 1;
                }
            })
        }

        for p in &self.items {
            if p.position.get_untracked() == 0 {
                p.panel_item.set(Some(new_item));
                break;
            }
        }
    }

    fn find_item_in_panel(
        &self,
        new_item: &RunsPanelItem,
    ) -> Option<RwSignal<Option<RunsPanelItem>>> {
        for p in &self.items {
            if p.panel_item
                .with_untracked(|i| i.as_ref().is_some_and(|i| i.id == new_item.id))
            {
                return Some(p.panel_item);
            }
        }
        return None;
    }

    fn was_pushed_before(&self, item: &RunsPanelItem) -> bool {
        self.push_ids.get_untracked().contains(&item.id)
    }

    pub fn push(&self, new_item: RunsPanelItem) {
        if let Some(item) = self.find_item_in_panel(&new_item) {
            item.set(Some(new_item));
        } else {
            if !self.was_pushed_before(&new_item) {
                self.push_ids.update_untracked(|x| {
                    x.insert(new_item.id);
                });
                self.append(new_item);
            }
        }
    }

    pub fn placements_for_sede(&self, sede: Signal<Arc<Sede>>) -> Signal<Vec<i64>> {
        let signals = self.items.iter().cloned().collect_vec();
        let memo_signals = signals.clone();
        Memo::new(move |_| {
            sede.with(|sede| {
                memo_signals
                    .iter()
                    .filter_map(|p| {
                        let panel_item = p.panel_item.get()?;
                        let id = panel_item.id;
                        let placement = panel_item.placement;

                        sede.team_belongs_str(&panel_item.team_login)
                            .then_some((placement, id))
                    })
                    .sorted_by_key(|(placement, _)| *placement)
                    .map(|(_, id)| id)
                    .collect_vec()
            })
        })
        .into()
    }
}
