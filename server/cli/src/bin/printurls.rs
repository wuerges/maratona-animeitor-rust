use clap::Parser;
use cli::configuration::{EventConfig, ServerConfig};
use service::event_store::deployment_site_key;
use service::revelation::{revelation_url, scoreboard_url};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Print scoreboard and revelation URLs offline from configuration"
)]
struct Args {
    #[arg(long)]
    event_config: PathBuf,
    #[arg(long)]
    server_config: PathBuf,
}
fn main() -> color_eyre::eyre::Result<()> {
    let args = Args::parse();
    let event = EventConfig::load(&args.event_config)?;
    let server = ServerConfig::load(&args.server_config)?;
    for contest in &event.contests {
        let url = scoreboard_url(
            &server.public_url.parse()?,
            &event.event.name,
            &contest.name,
        );
        println!(
            "-> {} / {}\n    Animeitor em {url}",
            event.event.name, contest.name
        );
        for site in &contest.sites {
            let key = deployment_site_key(
                &server.revelation_salt,
                &event.event.name,
                &contest.name,
                &site.name,
                &event.event.secret,
                &contest.secret,
                &site.secret,
            );
            let reveal = revelation_url(&url, &site.name, &key);
            println!("    {}: Reveleitor em {reveal}", site.name);
        }
    }
    Ok(())
}
