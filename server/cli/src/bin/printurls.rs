use clap::Parser;
use cli::configuration::{EventConfig, ServerConfig};
use service::event_store::deployment_site_key;
use std::path::PathBuf;
use url::Url;

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
        let mut url = Url::parse(&server.public_url)?;
        url.path_segments_mut()
            .map_err(|_| color_eyre::eyre::eyre!("public_url must support paths"))?
            .clear()
            .extend(["animeitor", &event.event.name, &contest.name, ""]);
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
            let mut reveal = url.clone();
            reveal
                .query_pairs_mut()
                .append_pair("secret", &key)
                .append_pair("sede", &site.name);
            println!("    {}: Reveleitor em {reveal}", site.name);
        }
    }
    Ok(())
}
