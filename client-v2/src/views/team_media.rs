use std::sync::Arc;

use data::configdata::Sede;
use leptos::{ev, html::Audio, logging::log, prelude::*};

use crate::{
    api::{team_photo_location, team_sound_location},
    model::team_signal::TeamSignal,
};

use super::{
    global_settings::{use_global_settings, GlobalSettingsSignal},
    team_score_line::TeamScoreLine,
};

#[derive(Clone, Default, PartialEq, Eq)]
pub enum PhotoState {
    #[default]
    Hidden,
    Show(String),
}

#[derive(Clone)]
pub struct PhotoStateSignal {
    photo_state: RwSignal<PhotoState>,
}

pub fn provide_global_photo_state() {
    provide_context(PhotoStateSignal {
        photo_state: RwSignal::new(PhotoState::default()),
    })
}

pub fn use_global_photo_state() -> RwSignal<PhotoState> {
    use_context::<PhotoStateSignal>().unwrap().photo_state
}

impl PhotoState {
    pub fn clicked(&mut self, team_login: &str) {
        *self = match self {
            PhotoState::Show(old) => {
                if old != team_login {
                    PhotoState::Show(team_login.to_string())
                } else {
                    PhotoState::Hidden
                }
            }
            PhotoState::Hidden => PhotoState::Show(team_login.to_string()),
        }
    }
    pub fn hide(&mut self) {
        *self = PhotoState::Hidden
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

#[component]
fn TeamAudio(team_login: String) -> impl IntoView {
    let audio_ref = NodeRef::<Audio>::new();
    let settings = use_context::<GlobalSettingsSignal>().unwrap();
    let volume_login = team_login.clone();
    let volume_settings = Signal::derive(move || {
        settings.global.with(|g| {
            g.team_settings
                .get(&volume_login)
                .cloned()
                .unwrap_or_default()
        })
    });

    Effect::new(move |_| {
        let volume = volume_settings.with(|v| v.volume);
        if let Some(audio) = audio_ref.get() {
            audio.set_volume(volume as f64 / 100_f64);
        }
    });

    let global_autoplay = Signal::derive(move || settings.global.with(|g| g.autoplay));

    let autoplay = Signal::derive(move || {
        volume_settings.with(|s| s.autoplay.unwrap_or(global_autoplay.get()))
    });

    let handle_settings = settings.clone();
    let handle_login = team_login.clone();
    let handle = window_event_listener(ev::keydown, move |ev| match ev.code().as_str() {
        "KeyM" => {
            let autoplay = autoplay.get();
            handle_settings.update_team_settings(&handle_login, |s| s.autoplay = Some(!autoplay));
        }
        code => log!("ev code: {code}"),
    });
    on_cleanup(move || handle.remove());

    let mute = Signal::derive(move || settings.global.with(|g| g.mute));
    let show_audio_controls =
        Signal::derive(move || settings.global.with(|g| g.show_audio_controls));

    let settings = settings.clone();
    let control_login = team_login.clone();
    let controls = move || {
        let control_autoplay_settings = settings.clone();
        let control_volume_settings = settings.clone();
        let control_autoplay_team_login = control_login.clone();
        let control_volume_team_login = control_login.clone();
        (show_audio_controls.get()).then_some(view! {
            <div class="volume_controls">
                <div class="control">
                    <label>autoplay</label>
                    <input
                        type="checkbox"
                        prop:checked=autoplay
                        on:input=move |ev| control_autoplay_settings.update_team_settings(&control_autoplay_team_login, |s| s.autoplay = Some(event_target_checked(&ev)))
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
                            on:input=move |ev| control_volume_settings.update_team_settings(&control_volume_team_login, |s| s.volume = event_target_value(&ev).parse().unwrap_or_default())
                        />
                    </div>
                </div>
            </div>
        })
    };
    let audio = move || {
        (!mute.get() && autoplay.get()).then_some(view! {
            <audio
                node_ref=audio_ref
                src=team_sound_location(&team_login)
                onerror=onerror_sound()
                autoplay=autoplay.get()
            />
        })
    };

    view! {
        {controls}
        {audio}
    }
}

#[component]
pub fn TeamMedia(
    team_login: String,
    show: RwSignal<PhotoState>,
    team: Arc<TeamSignal>,
    titulo: Signal<Option<Arc<Sede>>>,
    local_placement: Signal<Option<usize>>,
    sede: Signal<Arc<Sede>>,
) -> impl IntoView {
    let foto_id = format!("foto_{}", team_login);
    let team_name = team.name.clone();
    let escola = team.escola.clone();

    let settings = use_global_settings();
    let background_color = Signal::derive(move || {
        settings
            .global
            .with(|g| g.team_background_color.clone())
            .unwrap_or_default()
    });

    let memo = Memo::new(move |_| show.get());

    move || {
        let team_login_click = team_login.clone();
        let team_details =
            settings
                .global
                .with(|g| g.team_details)
                .then_some(if team_name.contains(&escola) {
                    view! {
                        <div class="foto_team_label">
                            <div class="foto_team_name">{team_name.clone()} </div>
                        </div>
                    }
                    .into_any()
                } else {
                    view! {
                        <div class="foto_team_label">
                            <div class="foto_team_name">{team_name.clone()} </div>
                            <div class="foto_team_escola">{escola.clone()} </div>
                        </div>
                    }
                    .into_any()
                });
        match memo.get() {
            PhotoState::Hidden => None,
            PhotoState::Show(team_login) => {
                if team_login == team_login_click {
                    Some(view! {
                        <div class="foto" id={foto_id.clone()} style:background-color=background_color>
                            <img
                                class="foto_img"
                                src=team_photo_location(&team_login)
                                onerror=onerror_photo()
                                on:click=move |_| show.update(|s| s.clicked(&team_login_click))
                            />
                            {team_details}
                            <TeamScoreLine team=team.clone() is_center=false.into() titulo local_placement sede />
                            <TeamAudio team_login=team_login.clone() />
                        </div>
                    })
                } else {
                    None
                }
            }
        }
    }
}
