use clap::Parser;

use data::event::{ContestConfig, Envelope, EventState, SiteConfig};
use service::event_store::site_key;
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};
use url::Url;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Prints the contest and reveleitor URLs of an animeitor server, reading
/// events, contests, sites and salts from the internal API.
struct SimpleParser {
    /// The animeitor server url.
    #[clap(short = 's', long, default_value = "http://localhost:8000")]
    server: String,

    /// Token for the internal API (/internal).
    #[clap(short = 't', long)]
    token: String,

    /// Only print this event.
    #[clap(long)]
    event: Option<String>,

    /// The url prefix for the printed URLs.
    #[clap(long, default_value = "http://localhost:8080")]
    prefix: String,
}

/// Fetches an enveloped resource from the internal API.
async fn get<T: for<'a> serde::Deserialize<'a>>(
    client: &reqwest::Client,
    token: &str,
    url: &str,
) -> color_eyre::eyre::Result<T> {
    let envelope: Envelope<T> = client
        .get(url)
        .basic_auth("usuario", Some(token))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    envelope
        .data
        .ok_or_else(|| color_eyre::eyre::eyre!("resposta sem data: {url}"))
}

fn contest_url(prefix: &str, event: &str, contest: &str) -> color_eyre::eyre::Result<Url> {
    Ok(Url::parse(prefix)?.join(&format!("/animeitor/{event}/{contest}/"))?)
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    let SimpleParser {
        server,
        token,
        event,
        prefix,
    } = SimpleParser::parse();

    let client = reqwest::Client::new();

    let mut events: Vec<String> =
        get(&client, &token, &format!("{server}/internal/events")).await?;
    events.sort();
    if let Some(event) = &event {
        events.retain(|name| name == event);
        if events.is_empty() {
            color_eyre::eyre::bail!("evento {event} não existe");
        }
    }

    let mut found_any = false;
    for event in &events {
        let state: EventState =
            get(&client, &token, &format!("{server}/internal/events/{event}")).await?;

        let mut contests: Vec<ContestConfig> = get(
            &client,
            &token,
            &format!("{server}/internal/events/{event}/contests"),
        )
        .await?;
        contests.sort_by(|a, b| a.name.cmp(&b.name));

        for contest in &contests {
            let mut sites: Vec<SiteConfig> = get(
                &client,
                &token,
                &format!(
                    "{server}/internal/events/{event}/contests/{}/sites",
                    contest.name
                ),
            )
            .await?;
            sites.sort_by(|a, b| a.name.cmp(&b.name));

            println!("-> {event} / {}", contest.name);
            println!("    Animeitor em {}", contest_url(&prefix, event, &contest.name)?);

            for site in &sites {
                match site_key(
                    state.salt.as_deref(),
                    contest.salt.as_deref(),
                    site.salt.as_deref(),
                    &contest.name,
                    &site.name,
                ) {
                    Some(key) => {
                        let mut url = contest_url(&prefix, event, &contest.name)?;
                        url.query_pairs_mut()
                            .append_pair("secret", &key)
                            .append_pair("sede", &site.name);
                        println!("    {}: Reveleitor em {url}", site.name);
                    }
                    None => println!("    {}: revelação desabilitada (site sem salt)", site.name),
                }
            }
            found_any = true;
        }
    }

    if !found_any {
        println!("nenhum contest encontrado");
    }

    Ok(())
}
