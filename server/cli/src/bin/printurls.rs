use clap::Parser;
use cli::SimpleArgs;
use data::configdata::{Contest, Secret, Sede};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};
use url::Url;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Maratona Rustrimeitor Server
struct SimpleParser {
    #[clap(flatten)]
    args: SimpleArgs,

    /// The url prefix for the animeitor server.
    #[clap(long, default_value = "http://localhost:8080", required = true)]
    prefix: String,
}

fn print_sede(url_prefix: &str, sede: &Sede) -> color_eyre::eyre::Result<()> {
    let mut url = Url::parse(&format!("{url_prefix}"))?;
    url.query_pairs_mut().append_pair("sede", &sede.entry.name);

    println!("-> {}", sede.entry.name);
    println!("    Animeitor em {}", url.as_str());
    println!("    Filters = {:?}", sede.entry.codes);
    Ok(())
}

fn print_reveleitor(url_prefix: &str, sede: &Sede, secret: &str) -> color_eyre::eyre::Result<()> {
    let mut url = Url::parse(&format!("{url_prefix}"))?;
    url.query_pairs_mut()
        .append_pair("secret", &secret)
        .append_pair("sede", &sede.entry.name);

    println!("-> {}", sede.entry.name);
    println!("    Reveleitor em {}", url.as_str());
    println!("    Filters = {:?}", sede.entry.codes);
    Ok(())
}

fn print_urls(
    url_prefix: &str,
    contest: &Contest,
    config_secret: &Secret,
) -> color_eyre::eyre::Result<()> {
    print_sede(url_prefix, &contest.titulo)?;
    for (_secret, sede) in &contest.sedes {
        print_sede(url_prefix, sede)?;
    }

    for (secret, sede) in &config_secret.sedes_by_secret {
        print_reveleitor(url_prefix, sede, &secret)?;
    }
    Ok(())
}

fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    let SimpleParser {
        args,
        prefix: url_prefix,
    } = SimpleParser::parse();

    let map = args.into_contest_and_secret()?;

    for (_, (_, contest, config_secret)) in &map {
        print_urls(&url_prefix, contest, config_secret)?;
    }

    Ok(())
}
