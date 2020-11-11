// extern crate diesel;

// use self::diesel::prelude::*;

// use turbineitor::models::*;
// use turbineitor::schema::*;
// use turbineitor::*;

use std::env;
use tokio;

use lib_server::*;
use turbineitor::Params;

// fn main() {

    // use runtable::dsl;

    // let connection = establish_connection();

    // let runs : i64 = dsl::runtable.count()
    //     .get_result(&connection)
    //     .expect("Error counting runs");

    // println!("Displaying {:?} runs", runs);
    // let results = dsl::runtable
    //     .limit(5)
    //     .load::<Runtable>(&connection)
    //     .expect("Error loading runs");
    // for run in results {
    //     println!("{}", run.usernumber);
    //     println!("----------\n");
    //     println!("{}", run.runproblem);
    // }


    // println!("All Runs -> {:?}", helpers::get_all_runs(&connection));
    // println!("Contest Data -> {:?}", helpers::get_contest_file(&connection));


// }

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
    
    let params = Params { contest_number, site_number };
    
    turbineitor::serve_simple_contest(server_port, &secret, params).await;

}