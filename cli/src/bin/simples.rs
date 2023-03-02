use data::configdata::ConfigSecret;
use server::{config::ServerConfig, *};

extern crate clap;
use clap::{App, Arg};
use url::Url;

#[tokio::main]
async fn main() -> eyre::Result<()> {
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
            Arg::with_name("public_port")
                .long("public")
                .value_name("PUBLIC")
                .help("Sets the public port for the server.")
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
            Arg::with_name("photos_path")
                .long("photos")
                .help("Path for the team photos")
                .value_name("PATH")
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

    let boca_url = matches.value_of("URL").expect("Expected an URL");
    let config_file = matches.value_of("config").unwrap_or("config/Default.toml");

    let config_file_escolas = matches.value_of("schools").unwrap_or("config/Escolas.toml");

    let config_file_teams = matches.value_of("teams").unwrap_or("config/Teams.toml");

    let config_sedes = config::parse_config(std::path::Path::new(config_file))
        .expect("Should be able to parse the config.");

    let config_escolas = config::parse_config(std::path::Path::new(config_file_escolas))
        .expect("Should be able to parse the config.");

    let config_teams = config::parse_config(std::path::Path::new(config_file_teams))
        .expect("Should be able to parse the config.");

    let config_secret = match matches.value_of("secret") {
        Some(path) => config::parse_config::<ConfigSecret>(std::path::Path::new(path))?,
        None => ConfigSecret::default(),
    }
    .get_patterns(&config_sedes);

    let config = config::pack_contest_config(config_sedes, config_escolas, config_teams);

    let hostname = matches.value_of("host");
    let public_port = matches
        .value_of("public_port")
        .and_then(|port| port.parse::<u16>().ok());

    let photos_path = std::path::Path::new(matches.value_of("photos_path").unwrap_or("photos"));
    if !photos_path.exists() {
        return Err(eyre::eyre!("path does not exists: {photos_path:?}"));
    }

    println!("-> Runs em http://localhost:{}/runspanel.html", server_port);
    println!(
        "-> Placar automatizado em http://localhost:{}/automatic.html",
        server_port
    );
    println!("-> Timer em http://localhost:{}/timer.html", server_port);
    println!(
        "-> Painel geral em http://localhost:{}/everything.html",
        server_port
    );
    println!(
        "-> Fotos dos times em http://localhost:{}/teams.html",
        server_port
    );
    println!(
        "-> Painel geral com sedes em http://localhost:{}/everything2.html",
        server_port
    );
    for (secret, sede) in config_secret.parameters.iter() {
        let mut url = Url::parse("http://localhost/reveleitor.html")?;
        url.set_host(hostname).ok();
        url.set_port(public_port).ok();
        url.query_pairs_mut()
            .append_pair("secret", secret)
            .append_pair("sede", &sede.name);

        println!("-> {}", sede.name);
        println!("    Reveleitor em {}", url.as_str());
        println!("    Filters = {:?}", sede.codes);
    }

    println!("\nMaratona Rustreimator rodando!");
    let server_config = ServerConfig {
        port: server_port,
        photos_path,
    };

    serve_simple_contest(config, boca_url.to_string(), config_secret, server_config).await;

    Ok(())
}
