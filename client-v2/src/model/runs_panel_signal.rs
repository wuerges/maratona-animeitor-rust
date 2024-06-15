use data::RunsPanelItem;
use itertools::Itertools;
use leptos::{create_rw_signal, RwSignal, SignalGetUntracked, SignalSet, SignalUpdate};

pub struct RunPanelItemSignal {
    pub panel_item: RwSignal<Option<RunsPanelItem>>,
    pub position: RwSignal<usize>,
}

pub struct RunsPanelItemManager {
    pub items: Vec<RunPanelItemSignal>,
}

const MAX: usize = 29;

impl RunsPanelItemManager {
    pub fn new() -> Self {
        Self {
            items: (0..=MAX)
                .map(|i| RunPanelItemSignal {
                    panel_item: create_rw_signal(None),
                    position: create_rw_signal(i),
                })
                .collect_vec(),
        }
    }

    pub fn push(&self, new_item: RunsPanelItem) {
        for p in &self.items {
            p.position.update(|i| {
                if *i == MAX {
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
}
