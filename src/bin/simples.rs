use tokio;

use server::*;

extern crate clap;
use clap::{App, Arg};

#[tokio::main]
async fn main() {
    let matches = App::new("Maratona Rustrimeitor Server")
        .version("0.1")
        .author("Emilio Wuerges. <wuerges@gmail.com>")
        .about("Runs the webserver hosting the rustrimeitor.")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG")
                .help("Sets a custom config file")
                .default_value("config/Default.toml")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("schools")
                .short("s")
                .long("schools")
                .value_name("SCHOOLS")
                .help("Sets a custom schools config file")
                .default_value("config/Escolas.toml")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("The TCP port to host the server")
                .default_value("8000")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("URL")
                .required(true)
                .help("The webcast url from BOCA.")
                .index(1),
        )
        .get_matches();

    println!("matches: {:?}", matches);

    let server_port: u16 = matches
        .value_of("port")
        .map(|p| p.parse().expect("Expected a TCP port"))
        .unwrap_or(8000);

    let url_base = matches.value_of("URL").unwrap();
    let config_file = matches.value_of("config").unwrap_or("config/Default.toml");

    let config_file_escolas = matches.value_of("schools").unwrap_or("config/Escolas.toml");

    let config_file_teams = matches.value_of("teams").unwrap_or("config/Teams.toml");

    let config_sedes = config::parse_config_sedes(std::path::Path::new(config_file))
        .expect("Should be able to parse the config.");

    let config_escolas = config::parse_config_escolas(std::path::Path::new(config_file_escolas))
        .expect("Should be able to parse the config.");
    
    let config_teams = config::parse_config_teams(std::path::Path::new(config_file_teams))
        .expect("Should be able to parse the config.");
    
    let config = config::pack_contest_config(config_sedes, config_escolas, config_teams);

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

    serve_simple_contest(config, url_base.to_string(), server_port, &secret).await;
}
