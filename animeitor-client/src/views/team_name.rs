use leptos::prelude::*;

#[component]
pub fn TeamName(escola: String, name: String) -> impl IntoView {
    let is_long = name.len() > 30;
    let show_escola = !name.contains(&escola);
    view! {
        <div class="cell time">
            <div class:nomeTime=true class:longTeamName=is_long >{name}</div>
            {show_escola.then(|| view! {
                <div class:nomeEscola=true >{escola}</div>
            })}
        </div>
    }
}
