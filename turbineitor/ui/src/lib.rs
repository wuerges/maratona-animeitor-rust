// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

pub mod requests;
pub mod views;

// use maratona_animeitor_rust::auth::UserKey;
use seed::{prelude::*, *};

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    Model {
        page: Page::Login {
            login: ElRef::new(),
            password: ElRef::new(),
        },
    }
    // Model {
    //     page: Page::Logged {
    //         login: "kappa".to_string(),
    //         // token: "aoeuaoeuaoeuoeau".to_string(),
    //         page : Internal::Problems,
    //     },
    // }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
#[derive(Debug)]
struct Model {
    page: Page,
}

type Input = ElRef<web_sys::HtmlInputElement>;

#[derive(Debug)]
enum Page {
    Login {
        login: Input,
        password: Input,
    },
    Logged {
        ws: WebSocket,
        login: String,
        page: Internal,
    },
}

#[derive(Debug)]
pub enum Internal {
    Pre,
    Basic,
    Problems,
    Clarifications,
    Scoreboard,
}

impl Page {
    fn login() -> Self {
        Page::Login {
            login: ElRef::new(),
            password: ElRef::new(),
        }
    }
    fn goto(&mut self, intern: Internal) {
        match self {
            Page::Logged { page, .. } => {
                *page = intern;
            }
            _ => (),
        }
    }
}

// ------ ------
//    Update
// ------ ------

// (Remove the line below once any of your `Msg` variants doesn't implement `Copy`.)
// #[derive(Clone)]
// `Msg` describes the different events you can modify state with.
pub enum Msg {
    Login(Input, Input),
    DoLogin(String, String),
    WS(WebSocketMessage),
    Logout,
    Goto(Internal),
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Login(login, password) => {
            let login = login.get().expect("login").value();
            let password = password.get().expect("password").value();

            let plogin = login.clone();

            let ws = WebSocket::builder(requests::get_ws_url("/sign"), orders)
                .on_message(Msg::WS)
                .on_open(move || Msg::DoLogin(login, password))
                .on_close(move |e| {
                    log!("websocket closed!", e);
                    Msg::Logout
                })                    
                .build_and_open()
                .expect("Open WebSocket");
            model.page = Page::Logged {
                ws,
                login: plogin,
                page: Internal::Pre,
            };
        }
        Msg::DoLogin(login, password) => {
            if let Page::Logged {
                ws,
                page: Internal::Pre,
                ..
            } = &model.page
            {
                ws.send_json(&data::auth::Credentials { login, password })
                    .expect("Should be able to send login and password");
            }
        }
        Msg::Logout => {
            model.page = Page::login();
        }
        Msg::Goto(intern) => {
            model.page.goto(intern);
        }
        Msg::WS(m) => {
            match m.json().expect("Should decode message do json") {
                data::turb::Msg::Ready => {}
                data::turb::Msg::Login => {
                    model.page.goto(Internal::Problems);
                }
                data::turb::Msg::Logout => {
                    model.page = Page::login();
                }
            }
            log!("received websocket message:", m);
        }
    }
}

// ------ ------
//     View
// ------ ------

// (Remove the line below once your `Model` become more complex.)
#[allow(clippy::trivially_copy_pass_by_ref)]
// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
    match &model.page {
        Page::Login { login, password } => {
            views::view_login_screen(login.clone(), password.clone())
        }
        Page::Logged { login, page, .. } => match page {
            Internal::Pre => section![
                C!["section"],
                div![
                    C!["container"],
                    p![C!["title"], "Checking credentials..."],
                    progress![C!["progress", "is-large", "is-info"], "60%"]
                ]
            ],
            _ => div![views::navbar(&login, &page),],
        },
    }
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
