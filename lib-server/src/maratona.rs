use std::env;
use tokio;
use warp::Filter;

use itertools::Itertools;

use maratona_animeitor_rust::{config, configdata};

use lib_server::*;

fn serve_contest(url_base : String, contest: &configdata::Contest, salt: &str, secret : &String)
 -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {

    let static_assets = warp::path("static").and(warp::fs::dir("static"));
    let seed_assets = warp::path("seed").and(warp::fs::dir("lib-seed"));

    let s = contest.sedes.iter()
        .map( |sede| &sede.source)
        .unique()
        .map( |source|  serve_urlbase(url_base.clone(), source, salt, secret))
        .fold1(|routes, r| r.or(routes).unify().boxed()).unwrap();

    static_assets.or(seed_assets).or(s)
}



#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Expected 3 arguments: {:?}", args);
        return;
    }
    let server_port :u16= match args[1].parse() {
        Ok(t) => t,
        Err(e) => panic!("Could not parse port {}", e),
    };
    let url_base = args[2].clone();
    let salt = args[3].clone();

    let secret = random_path_part();
    let routes = serve_contest(url_base, &config::contest(), &salt, &secret);
 
    println!("Reveleitor secret: {}", secret);
    warp::serve(routes).run(([0, 0, 0, 0], server_port)).await;    
}


