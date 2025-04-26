use leptos::prelude::*;

#[component]
pub fn TeamName(escola: String, name: String) -> impl IntoView {
    let isLong = name.len() > 30;
    view! {
        <div class="cell time">
            <div class:nomeEscola=true >{escola}</div>
            <div class:nomeTime=true class:longTeamName=isLong >{name}</div>
        </div>
    }
}
