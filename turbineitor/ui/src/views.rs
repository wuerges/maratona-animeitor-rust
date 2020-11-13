use maratona_animeitor_rust::auth::UserKey;
use seed::{prelude::*, *};

use crate::*;

pub fn view_login_screen() -> Node<Msg> {
    div![
        C!["modal", "is-active"],
        div![C!["modal-background"]],
        div![
            C!["modal-card"],
            header![
                C!["modal-card-head"],
                p![C!["modal-card-title"], "Preencha suas credenciais"]
            ],
            section![
                C!["modal-card-body"],
                div![
                    C!["field"],
                    div![C!["label"], "Login:"],
                    p![
                        C!["control", "has-icons-left"],
                        input![
                            C!["input"],
                            attrs! {At::Type=>"login", At::Placeholder=>"Login"}
                        ],
                        span![C!["icon", "is-small", "is-left"], "👤"],
                    ]
                ],
                div![
                    C!["field"],
                    div![C!["label"], "Senha:"],
                    p![
                        C!["control", "has-icons-left"],
                        input![
                            C!["input"],
                            attrs! {At::Type=>"password", At::Placeholder=>"Senha"}
                        ],
                        span![C!["icon", "is-small", "is-left"], "🔒"],
                    ]
                ]
            ],
            footer![
                C!["modal-card-foot"],
                button![C!["button", "is-success"], "Login"]
            ]
        ]
    ]
}
