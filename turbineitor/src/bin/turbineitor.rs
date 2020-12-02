use std::env;
use tokio;

use ::server::*;
use turbineitor::Params;

extern crate clap;
use clap::{App, Arg};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Expected 3 arguments: {:?}", args);
        return;
    }
    let server_port: u16 = match args[1].parse() {
        Ok(t) => t,
        Err(e) => panic!("Could not parse port {}", e),
    };
    let contest_number: i32 = match args[2].parse() {
        Ok(t) => t,
        Err(e) => panic!("Could not parse BOCA contest_number {}", e),
    };
    let site_number: i32 = match args[3].parse() {
        Ok(t) => t,
        Err(e) => panic!("Could not parse BOCA site_number {}", e),
    };
    
    let secret = random_path_part();
    let params = std::sync::Arc::new(Params::new(contest_number, site_number, secret));
    println!("Serving turbineitor data.");

    turbineitor::serve_turbinator_data(server_port, params).await
}
