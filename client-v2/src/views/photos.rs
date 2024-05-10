use leptos::{component, prelude::*, view, IntoView};

#[component]
fn TeamPhoto(team_login: String, show: RwSignal<bool>) -> impl IntoView {
    let foto_id = format!("foto_{}", team_login);
    view! {
        <div class="foto_img">
            <img src={team_photo_location(&team_login)} />
        </div>

        // div![id!["foto_container"],
        //     contest.teams.iter().map(|(team_login, team_entry)| {
        //         div![C!["foto"], id![foto_id],
        //             attrs!{At::OnClick =>
        //                 std::format!("document.getElementById('foto_{}').style.display = 'none';",
        //                 &team_entry.login)
        //             },
        //             // div![C!["nomeTime"], &team_entry.name],
        //             img![C!["foto_img"],
        //                 attrs!{At::Src => team_photo_location(team_login)},
        //                 attrs!{At::OnError => fake()}
        //             ],
        //         ]
        //     }),
    }
}
