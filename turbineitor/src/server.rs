extern crate itertools;
extern crate rand;

// use hyper::Client;
// use hyper_tls::HttpsConnector;

// use hyper::body;
// use std::io::prelude::*;
// use std::sync::Arc;
// use tokio;
// use tokio::{spawn, sync::Mutex};

// use serde::{Deserialize, Serialize};
use serde::Serialize;

use warp::Filter;

use std::collections::HashMap;

use crate::errors::Error;
use crate::helpers::*;
use crate::*;

use warp::reject::custom;

pub async fn serve_everything() {
    let secret = random_path_part();

    let params = Params {
        contest_number: 1,
        site_number: 1,
        secret,
        pool : establish_pool(),
    };
    let params_sign = params.clone();
    let sign_route = warp::post()
        .and(warp::path("sign"))
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .and_then(move |m| serve_sign(m, params_sign.clone()));

    let params_runs = params.clone();
    let runs_route = warp::path("runs")
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .and_then(move |m| auth_and_serve(m, params_runs.clone(), get_all_runs));


    let ui_route = warp::get().and(warp::fs::dir("turbineitor/ui"));

    let all_routes = ui_route.or(sign_route).or(runs_route);

    let server_port: u16 = 3033;
    warp::serve(all_routes)
        .run(([0, 0, 0, 0], server_port))
        .await;
}

async fn serve_sign(
    data: HashMap<String, String>,
    params: Params,
) -> Result<impl warp::Reply, warp::Rejection> {
    let login = data
        .get("login")
        .ok_or(warp::reject::custom(Error::empty("login")))?;
    let pass = data
        .get("password")
        .ok_or(warp::reject::custom(Error::empty("password")))?;

    let u = check_password(&login, &pass, &params).map_err(custom)?;
        // .ok_or(warp::reject::custom(Error::WrongPassword))?;

    auth::sign_user_key(u, params.secret.as_ref()).map_err(custom)
}

async fn auth_and_serve<F, R: Serialize>(
    data: HashMap<String, String>,
    params: Params,
    serve_stuff: F,
) -> Result<impl warp::Reply, warp::Rejection>
where
    F: Fn(&Params) -> Result<R, Error>,
{
    let token = data
        .get("token")
        .ok_or(warp::reject::custom(Error::empty("token")))?;
    auth::verify_user_key(&token, &params).map_err(warp::reject::custom)?;

    let result = serve_stuff(&params).map_err(warp::reject::custom)?;

    serde_json::to_string(&result)
        .map_err(Error::JsonEncode)
        .map_err(warp::reject::custom)
}

pub fn random_path_part() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const PASSWORD_LEN: usize = 16;
    let mut rng = rand::thread_rng();
    (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
