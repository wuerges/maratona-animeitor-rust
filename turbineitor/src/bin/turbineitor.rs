use std::env;
use tokio;

use ::server::*;
use turbineitor::Params;

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
    let contest_number :i32= match args[2].parse() {
        Ok(t) => t,
        Err(e) => panic!("Could not parse BOCA contest_number {}", e),
    };
    let site_number :i32= match args[3].parse() {
        Ok(t) => t,
        Err(e) => panic!("Could not parse BOCA site_number {}", e),
    };

    let secret = random_path_part();

    
    println!("Maratona Rustreimator rodando!");
    println!(
        "-> Runs em http://localhost:{}/seed/runspanel.html",
        server_port
    );
    println!(
        "-> Placar automatizado em http://localhost:{}/seed/automatic.html",
        server_port
    );
    println!(
        "-> Placar interativo em http://localhost:{}/seed/stepping.html",
        server_port
    );
    println!(
        "-> Timer em http://localhost:{}/seed/timer.html",
        server_port
    );
    println!(
        "-> Painel geral em http://localhost:{}/seed/everything.html",
        server_port
    );
    println!(
        "-> Reveleitor em http://localhost:{}/seed/reveleitor.html?secret={}",
        server_port, secret
    );
    
    let params = std::sync::Arc::new(Params::new(contest_number, site_number, secret));
    
    turbineitor::serve_simple_contest(server_port, params).await

}