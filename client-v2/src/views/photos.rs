use leptos::{component, html::Audio, prelude::*, view, IntoView};

use crate::api::{team_photo_location, team_sound_location};

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

    fn style(self) -> &'static str {
        match self {
            PhotoState::Unloaded => "display: none;",
            PhotoState::Show => "display: block;",
            PhotoState::Hidden => "display: none;",
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
pub fn TeamPhoto(team_login: String, show: RwSignal<PhotoState>) -> impl IntoView {
    let foto_id = format!("foto_{}", team_login);

    move || match show.get() {
        PhotoState::Unloaded => None,
        state => Some(view! {
            <div class="foto" id={foto_id.clone()} style={state.style()}>
                <img
                    class="foto_img"
                    src=team_photo_location(&team_login)
                    onerror=onerror_photo()
                    on:click=move |_| show.update(|s| s.clicked())
                />
                <audio src=team_sound_location(&team_login) onerror=onerror_sound() autoplay />
            </div>
        }),
    }
}
