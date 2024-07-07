use data::{RunTuple, RunsPanelItem};
use itertools::Itertools;
use leptos::{
    create_rw_signal, RwSignal, SignalGetUntracked, SignalSet, SignalUpdate, SignalWithUntracked,
};

pub struct RunPanelItemSignal {
    pub panel_item: RwSignal<Option<RunsPanelItem>>,
    pub position: RwSignal<usize>,
}

pub struct RunsPanelItemManager {
    pub items: Vec<RunPanelItemSignal>,
}

impl RunsPanelItemManager {
    pub const MAX: usize = 29;
    pub fn new() -> Self {
        Self {
            items: (0..=Self::MAX)
                .map(|i| RunPanelItemSignal {
                    panel_item: create_rw_signal(None),
                    position: create_rw_signal(i),
                })
                .collect_vec(),
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

    pub fn push(&self, new_item: RunsPanelItem) {
        if let Some(item) = self.find_item_in_panel(&new_item) {
            item.set(Some(new_item));
        } else {
            self.append(new_item)
        }
    }

    pub fn position_in_last_submissions(runs: &Vec<RunTuple>) -> usize {
        let mut non_waits = 0;
        for (i, run) in runs.iter().enumerate().rev() {
            if !run.answer.is_wait() {
                non_waits += 1;
            }
            if non_waits > Self::MAX + 1 {
                return i;
            }
        }
        return 0;
    }
}
