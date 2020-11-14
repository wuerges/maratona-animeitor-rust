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
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    // Model {
    //     page: Page::Login {
    //         login: ElRef::new(),
    //         password: ElRef::new(),
    //     },
    // }
    Model {
        page: Page::Problems {
            login: "kappa".to_string(),
            token: "aoeuaoeuaoeuoeau".to_string(),
        },
    }
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
    Login { login: Input, password: Input },
    Problems { login: String, token: String },
}

impl Page {
    fn login() -> Self {
        Page::Login {
            login: ElRef::new(),
            password: ElRef::new(),
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
    Logout,
    Token(fetch::Result<String>, String),
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Login(login, password) => {
            let login = login.get().expect("login").value();
            let password = password.get().expect("password").value();
            log!("yay:", login, password);

            orders.perform_cmd(requests::make_login(login, password));
        },
        Msg::Logout => {
            model.page = Page::login();
        },
        Msg::Token(Ok(token), login) => {
            model.page = Page::Problems { token, login };
        },
        Msg::Token(Err(e), _) => {
            log!("error on login:", e);
        } // Msg::Increment => *model += 1,
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
        Page::Problems { login, token } => views::view_problem_screen(&login),
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
