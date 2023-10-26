use clap::{App, Arg};
use cli::parse_config;
use data::configdata::*;
use url::Url;

fn main() -> color_eyre::eyre::Result<()> {
    let matches = App::new("Maratona Rustrimeitor URLs")
        .version("0.1")
        .author("Emilio Wuerges. <wuerges@gmail.com>")
        .about("Runs the webserver hosting the rustrimeitor.")
        .arg(
            Arg::with_name("config")
                .short("s")
                .long("sedes")
                .required(true)
                .value_name("SEDES")
                .help("Configures the sites")
                .default_value("config/Default.toml")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("secret")
                .short("x")
                .long("secret")
                .required(true)
                .value_name("SECRET")
                .help("Sets the secret to the reveleitor url.")
                .default_value("config/Secret.toml")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("prefix")
                .short("p")
                .long("prefix")
                .value_name("URL_PREFIX")
                .required(true)
                .help("The url prefix for the animeitor server.")
                .default_value("http://localhost:8080")
                .takes_value(true),
        )
        .get_matches();

    let config_file = matches.value_of("config").unwrap_or("config/Default.toml");
    let config_sedes = parse_config::<ConfigContest>(std::path::Path::new(config_file))
        .expect("Should be able to parse the config.")
        .into_contest();

    let config_secret = match matches.value_of("secret") {
        Some(path) => parse_config::<ConfigSecret>(std::path::Path::new(path))?,
        None => ConfigSecret::default(),
    }
    .into_secret(&config_sedes);

    let url_prefix = matches.value_of("prefix").unwrap_or_default();

    for (_secret, sede) in &config_sedes.sedes {
        let mut url = Url::parse(&format!("{url_prefix}/everything2.html"))?;
        url.query_pairs_mut().append_pair("sede", &sede.entry.name);

        println!("-> {}", sede.entry.name);
        println!("    Animeitor em {}", url.as_str());
        println!("    Filters = {:?}", sede.entry.codes);
    }

    for (secret, sede) in &config_secret.sedes_by_secret {
        let mut url = Url::parse(&format!("{url_prefix}/reveleitor.html"))?;
        url.query_pairs_mut()
            .append_pair("secret", secret)
            .append_pair("sede", &sede.entry.name);

        println!("-> {}", sede.entry.name);
        println!("    Reveleitor em {}", url.as_str());
        println!("    Filters = {:?}", sede.entry.codes);
    }

    Ok(())
}
