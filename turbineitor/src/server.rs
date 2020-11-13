extern crate rand;
extern crate itertools;

use hyper::Client;
use hyper_tls::HttpsConnector;

use hyper::body;
use std::io::prelude::*;
use std::sync::Arc;
use tokio;
use tokio::{spawn, sync::Mutex};

use warp::Filter;

use std::collections::HashMap;

use crate::*;
use crate::helpers::*;
use crate::errors::Error;

// pub fn check_password(username_p: &str
//     , password_p :&str
//     , connection: &PgConnection
//     , params : &Params) -> Option<helpers::UserKey> {

pub async fn serve_everything() {
    let secret = random_path_part();

    let params = Params {
        contest_number : 1,
        site_number : 1,
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
    // .and(warp::Filter::with(params.clone()))
    .and_then(move |m| {
        serve_runs(m, params_runs.clone())
    });

    let all_routes = sign_route.or(runs_route);

    let server_port : u16 = 3033;
    warp::serve(all_routes).run(([0, 0, 0, 0], server_port)).await;
}


async fn serve_sign(data : HashMap<String, String>, params : Params) -> Result<impl warp::Reply, warp::Rejection> {
    let connection = establish_connection();
    let result = check_password(&data["login"], &data["password"], &connection, &params);

    println!("checked login and password: {:?}", result);

    let result = result.and_then(|u| auth::sign_user_key(u, params.secret.as_ref()).ok() );

    println!("served a signature: {:?}", result);

    match result { 
        None => Err(warp::reject::not_found()),
        Some(r) => Ok(r),
    }
}

async fn serve_runs(data : HashMap<String, String>, params : Params) -> Result<impl warp::Reply, warp::Rejection> {
    let token = data.get("token").ok_or(warp::reject::custom(Error::EmptyToken))?;
    auth::verify_user_key(&token, &params).map_err(warp::reject::custom)?;

    let connection = establish_connection();
    let result = get_all_runs(&params, &connection);
    
    serde_json::to_string(&result).map_err(Error::JsonEncode).map_err(warp::reject::custom)
}



async fn auth_and_serve<F>(data : HashMap<String, String>, params : Params, serve_stuff: F) 
-> Result<impl warp::Reply, warp::Rejection>
where F: Fn(&Params, &PgConnection)
{
    let token = data.get("token").ok_or(warp::reject::custom(Error::EmptyToken))?;
    auth::verify_user_key(&token, &params).map_err(warp::reject::custom)?;

    let connection = establish_connection();
    let result = serve_stuff(&params, &connection);
    
    serde_json::to_string(&result).map_err(Error::JsonEncode).map_err(warp::reject::custom)
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
