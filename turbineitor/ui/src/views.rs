use data::auth::UserKey;
use seed::{prelude::*, *};

use crate::*;

pub fn view_login_screen(
    login: ElRef<web_sys::HtmlInputElement>,
    password: ElRef<web_sys::HtmlInputElement>,
) -> Node<Msg> {
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
                            el_ref(&login),
                            attrs! {At::Type=>"login", At::Placeholder=>"Login"},
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
                            el_ref(&password),
                            attrs! {At::Type=>"password", At::Placeholder=>"Senha"}
                        ],
                        span![C!["icon", "is-small", "is-left"], "🔒"],
                    ]
                ]
            ],
            footer![
                C!["modal-card-foot"],
                button![
                    C!["button", "is-success"],
                    "Login",
                    ev(Ev::Click, move |_| Msg::Login(login, password))
                ],
            ]
        ]
    ]
}

pub fn navbar(login: &String, page: &Internal) -> Node<Msg> {
    nav![
        C!["navbar", "is-dark"],
        attrs! {"role"=>"navigation", At::AriaLabel=>"main navigation"},
        div![
            C!["navbar-brand"],
            div![
                C!["navbar-item"],
                img![
                    attrs! {At::Src=>"assets/titulo-1stPhase.svg", At::Width=>"200", At::Height=>"80"}
                ],
            ],
            a![
                C!["navbar-burger", "burger"],
                attrs! {"role"=>"button", At::AriaLabel=>"menu", At::AriaExpanded=>"false",
                "data-target"=>"navbarBasicExample",
                At::OnClick=>"document.querySelector('.navbar-menu').classList.toggle('is-active');"},
                span![attrs! {At::AriaHidden=>"true"}],
                span![attrs! {At::AriaHidden=>"true"}],
                span![attrs! {At::AriaHidden=>"true"}],
            ]
        ],
        div![
            id!["navbarBasicExample"],
            C!["navbar-menu"],
            div![
                C!["navbar-start"],
                a![
                    C![
                        "navbar-item",
                        IF!(matches!(page, Internal::Basic) => "is-active")
                    ],
                    "Informações Base",
                    ev(Ev::Click, |_| Msg::Goto(Internal::Basic))
                ],
                a![
                    C![
                        "navbar-item",
                        IF!(matches!(page, Internal::Problems) => "is-active")
                    ],
                    "Problemas",
                    ev(Ev::Click, |_| Msg::Goto(Internal::Problems))
                ],
                a![
                    C![
                        "navbar-item",
                        IF!(matches!(page, Internal::Clarifications) => "is-active")
                    ],
                    "Clarifications",
                    ev(Ev::Click, |_| Msg::Goto(Internal::Clarifications))
                ],
                a![
                    C![
                        "navbar-item",
                        IF!(matches!(page, Internal::Scoreboard) => "is-active")
                    ],
                    "Placar",
                    ev(Ev::Click, |_| Msg::Goto(Internal::Scoreboard))
                ],
            ],
            div![
                C!["navbar-end"],
                a![
                    C!["navbar-item"],
                    div![
                        C!["buttons"],
                        a![C!["button", "is-static"], strong![login]],
                        a![
                            C!["button", "is-danger"],
                            "Log out",
                            ev(Ev::Click, |_| Msg::Logout)
                        ],
                    ]
                ],
            ]
        ]
    ]
}
