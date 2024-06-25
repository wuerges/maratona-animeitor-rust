use leptos::{component, prelude::*, view, IntoView};

use crate::{
    api::{team_photo_location, team_sound_location},
    model::TeamSignal,
};

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

#[component]
pub fn TeamPhoto<'cs>(
    team_login: String,
    show: RwSignal<PhotoState>,
    team: &'cs TeamSignal,
) -> impl IntoView {
    let foto_id = format!("foto_{}", team_login);
    let team_name = team.name.clone();
    let escola = team.escola.clone();

    move || match show.get() {
        PhotoState::Unloaded => None,
        PhotoState::Hidden => None,
        PhotoState::Show => Some(view! {
            <div class="foto" id={foto_id.clone()}>
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
            </div>
            <audio src=team_sound_location(&team_login) onerror=onerror_sound() autoplay />
        }),
    }
}
