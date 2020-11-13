extern crate itertools;
extern crate rand;

use hyper::Client;
use hyper_tls::HttpsConnector;

use hyper::body;
use std::io::prelude::*;
use std::sync::Arc;
use tokio;
use tokio::{spawn, sync::Mutex};

use serde::{Deserialize, Serialize};

use warp::Filter;

use std::collections::HashMap;

use crate::errors::Error;
use crate::helpers::*;
use crate::*;

pub async fn serve_everything() {
    let secret = random_path_part();

    let params = Params {
        contest_number: 1,
        site_number: 1,
        secret,
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

    let all_routes = sign_route.or(runs_route);

    let server_port: u16 = 3033;
    warp::serve(all_routes)
        .run(([0, 0, 0, 0], server_port))
        .await;
}

async fn serve_sign(
    data: HashMap<String, String>,
    params: Params,
) -> Result<impl warp::Reply, warp::Rejection> {
    let connection = establish_connection();
    let login = data
        .get("login")
        .ok_or(warp::reject::custom(Error::empty("login")))?;
    let pass = data
        .get("password")
        .ok_or(warp::reject::custom(Error::empty("password")))?;

    let u = check_password(&login, &pass, &connection, &params)
        .ok_or(warp::reject::custom(Error::WrongPassword))?;

    auth::sign_user_key(u, params.secret.as_ref()).map_err(warp::reject::custom)
}

// async fn serve_runs(data : HashMap<String, String>, params : Params) -> Result<impl warp::Reply, warp::Rejection> {
//     let token = data.get("token").ok_or(warp::reject::custom(Error::EmptyToken))?;
//     auth::verify_user_key(&token, &params).map_err(warp::reject::custom)?;

//     let connection = establish_connection();
//     let result = get_all_runs(&params, &connection).map_err(warp::reject::custom)?;
//     serde_json::to_string(&result).map_err(Error::JsonEncode).map_err(warp::reject::custom)
// }

async fn auth_and_serve<F, R: Serialize>(
    data: HashMap<String, String>,
    params: Params,
    serve_stuff: F,
) -> Result<impl warp::Reply, warp::Rejection>
where
    F: Fn(&Params, &PgConnection) -> Result<R, Error>,
{
    let token = data
        .get("token")
        .ok_or(warp::reject::custom(Error::empty("token")))?;
    auth::verify_user_key(&token, &params).map_err(warp::reject::custom)?;

    let connection = establish_connection();
    let result = serve_stuff(&params, &connection).map_err(warp::reject::custom)?;

    serde_json::to_string(&result)
        .map_err(Error::JsonEncode)
        .map_err(warp::reject::custom)
}

// pub async fn server_everything() {
//     let routes = server::route_everything();

// }

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
