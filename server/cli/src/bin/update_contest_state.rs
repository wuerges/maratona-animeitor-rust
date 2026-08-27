use clap::Parser;

use cli::sentry;
use service::event_store::from_legacy_contest_state;
use service::webcast;
use tracing::{debug, error};
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Maratona Rustrimeitor Server
struct SimpleParser {
    /// Token for the internal API (/internal).
    #[clap(short = 't', long)]
    internal_token: String,

    /// The webcast url from BOCA.
    #[clap(short = 'i')]
    boca_url: String,

    /// The animeitor server url.
    #[clap(short = 's')]
    server_url: String,

    /// The event fed by this loop.
    #[clap(long, default_value = "default")]
    event: String,
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    let SimpleParser {
        internal_token,
        boca_url,
        server_url,
        event,
    } = SimpleParser::parse();

    tracing::info!("\nSetting up sentry guard");
    let _guard = sentry::setup();

    db_update_loop(&internal_token, &boca_url, &server_url, &event).await;

    Ok(())
}

pub async fn db_update_loop(internal_token: &str, boca_url: &str, server_url: &str, event: &str) {
    let dur = tokio::time::Duration::new(1, 0);
    let mut interval = tokio::time::interval(dur);

    let client = reqwest::Client::new();
    let event_url = format!("{server_url}/internal/events/{event}");
    let runs_url = format!("{event_url}/runs");

    loop {
        interval.tick().await;

        match webcast::load_data_from_url_maybe(boca_url).await {
            Ok(contest_state) => {
                let (state, runs) = from_legacy_contest_state(&contest_state, event);

                // The event may not exist yet on the first tick.
                let result = client
                    .put(&event_url)
                    .basic_auth("usuario", Some(internal_token))
                    .json(&state)
                    .send()
                    .await;
                match result {
                    Ok(response) => {
                        let status = response.status();
                        match response.error_for_status() {
                            Ok(_) => debug!("event updated"),
                            Err(_) if status == reqwest::StatusCode::NOT_FOUND => {
                                match client
                                    .post(&event_url)
                                    .basic_auth("usuario", Some(internal_token))
                                    .json(&state)
                                    .send()
                                    .await
                                {
                                    Ok(created) => match created.error_for_status() {
                                        Ok(_) => debug!("event created"),
                                        Err(err) => error!(?err, "status error creating event"),
                                    },
                                    Err(err) => error!(?err, "network error creating event"),
                                }
                            }
                            Err(err) => error!(?err, "status error updating event"),
                        }
                    }
                    Err(err) => error!(?err, "network error updating event"),
                }

                match client
                    .post(&runs_url)
                    .basic_auth("usuario", Some(internal_token))
                    .json(&serde_json::json!({ "runs": runs }))
                    .send()
                    .await
                {
                    Ok(result) => match result.error_for_status() {
                        Ok(_) => debug!("runs sent"),
                        Err(err) => error!(?err, "status error sending runs"),
                    },
                    Err(err) => error!(?err, "network error sending runs"),
                }
            }
            Err(err) => error!(?err, "failed loading contest state from BOCA, will retry"),
        }
    }
}
