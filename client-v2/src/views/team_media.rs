use data::configdata::Sede;
use leptos::{
    component, create_effect, create_node_ref, event_target_checked, event_target_value,
    html::Audio, prelude::*, use_context, view, IntoView,
};
use leptos_use::{storage::use_local_storage, utils::JsonCodec};
use serde::{Deserialize, Serialize};

use crate::{
    api::{team_photo_location, team_sound_location},
    model::TeamSignal,
};

use super::{
    global_settings::{use_global_settings, GlobalSettingsSignal},
    team_score_line::TeamScoreLine,
};

use std::rc::Rc;

#[derive(Clone, Copy, Default)]
pub enum PhotoState {
    #[default]
    Unloaded,
    Show,
    Hidden,
}

impl PhotoState {
    pub fn clicked(&mut self) {
        *self = match self {
            PhotoState::Unloaded => PhotoState::Show,
            PhotoState::Show => PhotoState::Hidden,
            PhotoState::Hidden => PhotoState::Show,
        }
    }
}

fn onerror_photo() -> String {
    format!(
        "this.onerror=null; this.src='{}'",
        team_photo_location("fake")
    )
}

fn onerror_sound() -> String {
    format!(
        "this.onerror=null; this.src='{}'",
        team_sound_location("applause")
    )
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct VolumeSettings {
    autoplay: bool,
    volume: u32,
}

impl Default for VolumeSettings {
    fn default() -> Self {
        Self {
            autoplay: true,
            volume: 100,
        }
    }
}

#[component]
fn TeamAudio(team_login: String) -> impl IntoView {
    let key = format!("volume.{}", team_login);

    let (volume_settings, set_volume_settings, _) =
        use_local_storage::<VolumeSettings, JsonCodec>(&key);

    let audio_ref = create_node_ref::<Audio>();

    create_effect(move |_| {
        let volume = volume_settings.with(|v| v.volume);
        if let Some(audio) = audio_ref.get() {
            audio.set_volume(volume as f64 / 100_f64);
        }
    });

    let settings = use_context::<GlobalSettingsSignal>().unwrap();

    let autoplay = move || volume_settings.with(|s| s.autoplay);
    let mute = (move || settings.global.with(|g| g.mute)).into_signal();

    let should_autoplay = move || !mute.get() && autoplay();

    move || {
        (!mute.get()).then_some(view! {
            <div class="volume_controls">
                <div class="control">
                    <label>autoplay</label>
                    <input
                        type="checkbox"
                        prop:checked=autoplay
                        on:input=move |ev| set_volume_settings.update(|v| v.autoplay = event_target_checked(&ev))
                    />
                </div>
                <div class="control">
                    <label>volume</label>
                    <div class="volume_slide">
                        <input
                            type="range"
                            min="0" max="100"
                            value="100"
                            prop:value=move || volume_settings.with(|v| v.volume)
                            on:input=move |ev| set_volume_settings.update(|v| v.volume = event_target_value(&ev).parse().unwrap_or_default())
                        />
                    </div>
                </div>
            </div>
            <audio
                ref=audio_ref
                src=team_sound_location(&team_login)
                onerror=onerror_sound()
                autoplay=should_autoplay
            />
        })
    }
}

#[component]
pub fn TeamMedia(
    team_login: String,
    show: RwSignal<PhotoState>,
    team: Rc<TeamSignal>,
    titulo: Signal<Option<Rc<Sede>>>,
    local_placement: Signal<Option<usize>>,
    sede: Signal<Rc<Sede>>,
) -> impl IntoView {
    let foto_id = format!("foto_{}", team_login);
    let team_name = team.name.clone();
    let escola = team.escola.clone();

    let settings = use_global_settings();
    let background_color = move || settings.global.with(|g| g.background_color.clone());

    move || match show.get() {
        PhotoState::Unloaded => None,
        PhotoState::Hidden => None,
        PhotoState::Show => Some(view! {
            <div class="foto" id={foto_id.clone()} style:background-color=background_color>
                <img
                    class="foto_img"
                    src=team_photo_location(&team_login)
                    onerror=onerror_photo()
                    on:click=move |_| show.update(|s| s.clicked())
                />
                <div class="foto_team_label">
                    <div class="foto_team_name">{team_name.clone()} </div>
                    <div class="foto_team_escola">{escola.clone()} </div>
                </div>
                <TeamScoreLine team=team.clone() is_center=(|| false).into() titulo local_placement sede />
                <TeamAudio team_login=team_login.clone() />
            </div>
        }),
    }
}
