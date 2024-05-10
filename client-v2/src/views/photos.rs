use leptos::{component, prelude::*, view, IntoView};

use crate::api::team_photo_location;

#[component]
pub fn TeamPhoto(team_login: String, show: RwSignal<bool>) -> impl IntoView {
    let foto_id = format!("foto_{}", team_login);
    let style = move || if show.get() { "" } else { "display: none;" };
    view! {
        <div class="foto" id={foto_id} style={style}>
            <img
                class="foto_img"
                src={team_photo_location(&team_login)}
                onerror={format!("this.onerror=null; this.src='{}'", team_photo_location("fake"))}
            />
        </div>
    }
}
