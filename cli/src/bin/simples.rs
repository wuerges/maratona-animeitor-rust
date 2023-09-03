use cli::parse_config;
use data::configdata::ConfigSecret;
use server::{config::ServerConfig, *};

extern crate clap;
use clap::{App, Arg};

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
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
            Arg::with_name("secret")
                .short("x")
                .long("secret")
                .value_name("SECRET")
                .help("Sets the secret to the reveleitor url.")
                .default_value("config/Secret.toml")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("HOST")
                .help("Sets the hostname for the server.")
                .default_value("localhost")
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

    let server_port: u16 = matches
        .value_of("port")
        .map(|p| p.parse().expect("Expected a TCP port"))
        .unwrap_or(8000);

    let boca_url = matches.value_of("URL").expect("Expected an URL");
    let config_file = matches.value_of("config").unwrap_or("config/Default.toml");

    let config_sedes = parse_config(std::path::Path::new(config_file))
        .expect("Should be able to parse the config.");

    let config_secret = match matches.value_of("secret") {
        Some(path) => parse_config::<ConfigSecret>(std::path::Path::new(path))?,
        None => ConfigSecret::default(),
    }
    .get_patterns(&config_sedes);

    let config = config::pack_contest_config(config_sedes);

    let server_config = ServerConfig { port: server_port };

    println!("\nSetting up sentry guard");
    let _guard = sentry::setup();
    let _autometrics = metrics::setup();

    println!("\nMaratona Rustreimator rodando!");
    serve_simple_contest(config, boca_url.to_string(), config_secret, server_config).await;

    Ok(())
}
