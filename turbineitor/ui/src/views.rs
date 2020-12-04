// use data::auth::UserKey;
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

use data::turb::ClarificationSet;

pub fn view_clarifications(selected: &String, groups: &ClarificationSet) -> Node<Msg> {
    div![
        C!["column"],
        section![
            C!["section"],
            div![
                C!["tabs"],
                ul![groups
                    .clars
                    .iter()
                    .map(|(_, g)| li![a![&g.name], "(", g.len(), ")"])]
            ],
            div![
                C!["content"],
                h5![C!["title", "is-5", "is-primary"], "Clarifications:"],
                // script![
                //     "function toggleModal() {
                //          document.querySelector('.modal').classList.toggle('is-active');
                // }"
                // ],
                div![
                    C!["block"],
                    a![
                        C!["button", "is-link"],
                        attrs! {At::OnClick=>"toggleModal();"},
                        "Fazer novo clarification "
                    ],
                ],
                div![
                    C!["modal"],
                    div![C!["modal-background"]],
                    div![
                        C!["modal-card"],
                        header![
                            C!["modal-card-head"],
                            p![C!["modal-card-title"], "Novo clarification:"],
                        ],
                        section![
                            C!["modal-card-body"],
                            textarea![
                                C!["textarea"],
                                attrs! {At::Placeholder=>"Escreva aqui sua pergunta."}
                            ],
                        ],
                        footer![
                            C!["modal-card-foot"],
                            button![C!["button", "is-success"], "Enviar"],
                            button![
                                C!["button"],
                                attrs! {At::OnClick=>"toggleModal();"},
                                "Cancelar"
                            ],
                        ],
                    ]
                ],
                groups
                    .get(selected).unwrap_or(&data::turb::ClarificationGroup::new(selected.clone()))
                    .clarifications.iter().map(|c| div![
                        C!["box"],
                        p![C!["title", "is-5"], "⏱", c.time],
                        div![
                            C!["columns"],
                            div![
                                C!["column"],
                                p![C!["title", "is-5"], "Pergunta:"],
                                p![&c.question],
                            ],
                            div![
                                C!["column"],
                                p![C!["title", "is-5"], "Resposta:"],
                                p![&c.answer],
                            ],
                        ]
                    ])
            ],
        ],
    ]
}

use data::turb::{Ans, RunSet};

fn check_mark(a: &Ans) -> &'static str {
    match a {
        Ans::Yes => "✔",
        Ans::No => "✗",
        Ans::Wait => "☐",
    }
}

fn check_style(a: &Ans) -> seed::Attrs {
    match a {
        Ans::Yes => C!["is-success"],
        Ans::No => C!["is-danger"],
        Ans::Wait => C!["is-warning", "is-loading"],
    }
}

pub fn view_submissions(runs: &RunSet) -> Node<Msg> {
    aside![
        C!["column", "is-3"],
        section![
            C!["section"],
            h5![C!["title", "is-5", "is-primary"], "Submissões:"],
            div![
                C!["box"],
                runs.runs.iter().map(|(_, r)| div![
                    C!["columns", "is-vcentered", "tags", "has-addons"],
                    div![C!["column", "tag"], &r.problem],
                    div![C!["column", "tag"], r.time],
                    div![C!["column", "tag"], &r.language],
                    div![
                        C!["column", "tag"],
                        check_style(&r.result),
                        check_mark(&r.result)
                    ],
                ])
            ],
        ],
    ]
}
