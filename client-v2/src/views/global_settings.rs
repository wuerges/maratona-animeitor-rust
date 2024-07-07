use leptos::{provide_context, Signal, WriteSignal};
use leptos_use::{storage::use_local_storage, utils::JsonCodec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub autoplay: bool,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self { autoplay: true }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalSettingsSignal {
    pub global: Signal<GlobalSettings>,
    pub set_global: WriteSignal<GlobalSettings>,
}

pub fn provide_global_settings() {
    let (get, set, _) = use_local_storage::<GlobalSettings, JsonCodec>("global.settings");

    provide_context(GlobalSettingsSignal {
        global: get,
        set_global: set,
    })
}
