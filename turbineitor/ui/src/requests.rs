// use data::auth::*;
// use seed::{prelude::*, *};
use seed::prelude::*;
// use crate::*;

// pub async fn fetch_login(login: String, password: String) -> fetch::Result<String> {
//     Request::new("/sign")
//         .method(Method::Post)
//         .json(&Credentials { login, password })?
//         .fetch()
//         .await?
//         .check_status()?
//         .text()
//         .await
// }

// pub async fn make_login(login: String, password: String) -> Msg {
//     Msg::Token(fetch_login(login.clone(), password).await, login)
// }

pub fn get_ws_url(path :&str) -> String {
    let window = web_sys::window().expect("Should have a window");
    let location = window.location().href().expect("Should have a URL");
    let url = web_sys::Url::new(&location)
        .expect("Location should be valid");
    url.set_protocol("ws:");
    url.set_pathname(path);
    url.href()

}