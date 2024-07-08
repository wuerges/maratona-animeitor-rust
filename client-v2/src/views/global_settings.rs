use leptos::*;
use leptos_use::{storage::use_local_storage, utils::JsonCodec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub mute: bool,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self { mute: true }
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

#[component]
pub fn SettingsPanel() -> impl IntoView {
    let global = use_context::<GlobalSettingsSignal>().unwrap();

    view! {
        <div class="settings_panel">
            <div class="control">
            <label>mute</label>
                <input
                    type="checkbox"
                    prop:checked=move || global.global.with(|g| g.mute)
                    on:input=move |ev| global.set_global.update(|g| g.mute = event_target_checked(&ev))
                />
            </div>
        </div>
    }
}
