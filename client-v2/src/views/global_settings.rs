use leptos::{create_rw_signal, RwSignal};

#[derive(Debug, Clone)]
pub struct GlobalSettings {
    pub autoplay: RwSignal<bool>,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            autoplay: create_rw_signal(true),
        }
    }
}
