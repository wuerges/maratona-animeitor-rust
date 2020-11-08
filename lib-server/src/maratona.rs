use std::env;
use tokio;

use maratona_animeitor_rust::config;

use lib_server::*;



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


