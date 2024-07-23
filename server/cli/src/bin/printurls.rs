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

    /// Show filters.
    #[clap(long, default_value = "false")]
    filters: bool,
}

fn print_sede(
    parse: &SimpleParser,
    sede: &Sede,
    contest_name: &str,
) -> color_eyre::eyre::Result<()> {
    let mut url = Url::parse(&parse.prefix)?;
    if !contest_name.is_empty() {
        url.query_pairs_mut().append_pair("contest", contest_name);
    }
    url.query_pairs_mut().append_pair("sede", &sede.entry.name);

    println!("-> {}", sede.entry.name);
    println!("    Animeitor em {}", url.as_str());
    if parse.filters {
        println!("    Filters = {:?}", sede.entry.codes);
    }
    Ok(())
}

fn print_reveleitor(
    parse: &SimpleParser,
    sede: &Sede,
    secret: &str,
) -> color_eyre::eyre::Result<()> {
    let mut url = Url::parse(&parse.prefix)?;
    url.query_pairs_mut()
        .append_pair("secret", &secret)
        .append_pair("sede", &sede.entry.name);

    println!("-> {}", sede.entry.name);
    println!("    Reveleitor em {}", url.as_str());
    if parse.filters {
        println!("    Filters = {:?}", sede.entry.codes);
    }
    Ok(())
}

fn print_urls(
    parse: &SimpleParser,
    contest: &Contest,
    config_secret: &Secret,
    contest_name: &str,
) -> color_eyre::eyre::Result<()> {
    println!("\n");
    print_sede(parse, &contest.titulo, contest_name)?;
    for (_secret, sede) in &contest.sedes {
        print_sede(parse, sede, contest_name)?;
    }

    for (secret, sede) in &config_secret.sedes_by_secret {
        print_reveleitor(parse, sede, &secret)?;
    }
    Ok(())
}

fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    let parse = SimpleParser::parse();

    let map = parse.args.into_contest_and_secret()?;

    for (name, (_, contest, config_secret)) in &map {
        print_urls(&parse, contest, config_secret, name)?;
    }

    Ok(())
}
