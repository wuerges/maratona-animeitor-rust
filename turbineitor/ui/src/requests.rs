use data::auth::*;
use seed::{prelude::*, *};
use crate::*;

pub async fn fetch_login(login: String, password: String) -> fetch::Result<String> {
    Request::new("/sign")
        .method(Method::Post)
        .json(&Credentials { login, password })?
        .fetch()
        .await?
        .check_status()?
        .text()
        .await
}

pub async fn make_login(login: String, password: String) -> Msg {
    Msg::Token(fetch_login(login.clone(), password).await, login)
}