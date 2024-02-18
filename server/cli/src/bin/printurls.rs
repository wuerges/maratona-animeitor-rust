use clap::Parser;
use cli::SimpleArgs;
use url::Url;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Maratona Rustrimeitor Server
struct SimpleParser {
    #[clap(flatten)]
    args: SimpleArgs,

    /// The url prefix for the animeitor server.
    #[clap(
        short = 'p',
        long,
        default_value = "http://localhost:8080",
        required = true
    )]
    prefix: String,
}

fn main() -> color_eyre::eyre::Result<()> {
    let SimpleParser {
        args,
        prefix: url_prefix,
    } = SimpleParser::parse();

    let (_, contest, config_secret) = args.into_contest_and_secret()?;

    for (_secret, sede) in &contest.sedes {
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
