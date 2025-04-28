use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::storage::use_local_storage;
use serde::{Deserialize, Serialize};

use super::team_media::provide_global_photo_state;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub mute: bool,
    pub autoplay: bool,
    pub show_audio_controls: bool,
    pub background_color: Option<String>,
    pub team_background_color: Option<String>,
    pub secret_enabled: bool,
    pub secret: Option<String>,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            mute: true,
            show_audio_controls: false,
            background_color: None,
            team_background_color: None,
            secret_enabled: false,
            secret: None,
            autoplay: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalSettingsSignal {
    pub global: Signal<GlobalSettings>,
    pub set_global: WriteSignal<GlobalSettings>,
}

impl GlobalSettings {
    pub fn get_secret(&self) -> Option<String> {
        self.secret_enabled.then_some(self.secret.clone()).flatten()
    }
}

pub fn provide_global_settings() {
    let (get, set, _) = use_local_storage::<GlobalSettings, JsonSerdeCodec>("global.settings");

    provide_global_photo_state();
    provide_context(GlobalSettingsSignal {
        global: get,
        set_global: set,
    })
}

fn maybe_text(text: String) -> Option<String> {
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

pub fn use_global_settings() -> GlobalSettingsSignal {
    use_context::<GlobalSettingsSignal>().unwrap()
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
            <div class="control">
            <label>autoplay</label>
                <input
                    type="checkbox"
                    prop:checked=move || global.global.with(|g| g.autoplay)
                    on:input=move |ev| global.set_global.update(|g| g.autoplay = event_target_checked(&ev))
                />
            </div>
            <div class="control">
            <label>show audio controls</label>
                <input
                    type="checkbox"
                    prop:checked=move || global.global.with(|g| g.show_audio_controls)
                    on:input=move |ev| global.set_global.update(|g| g.show_audio_controls = event_target_checked(&ev))
                />
            </div>
            <div class="control">
                <label>background_color</label>
                <input
                    type="text"
                    prop:value=move || global.global.with(|g| g.background_color.clone().unwrap_or_default())
                    on:input=move |ev| global.set_global.update(|g| g.background_color = maybe_text(event_target_value(&ev)))
                />
            </div>
            <div class="control">
                <label>team_background_color</label>
                <input
                    type="text"
                    prop:value=move || global.global.with(|g| g.team_background_color.clone().unwrap_or_default())
                    on:input=move |ev| global.set_global.update(|g| g.team_background_color = maybe_text(event_target_value(&ev)))
                />
            </div>
            <div class="control">
            <label>secret</label>
            <input
                    type="checkbox"
                    prop:checked=move || global.global.with(|g| g.secret_enabled)
                    on:input=move |ev| global.set_global.update(|g| g.secret_enabled = event_target_checked(&ev))
                />
            <input
                type="password"
                prop:value=move || global.global.with(|g| g.secret.clone().unwrap_or_default())
                on:input=move |ev| global.set_global.update(|g| g.secret = maybe_text(event_target_value(&ev)))
            />
        </div>
        </div>
    }
}
