use clap::Parser;

use cli::sentry;
use service::event_store::{EventState, from_legacy_contest_state};
use service::webcast;
use tracing::{debug, error};
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Feeder: polls BOCA and publishes the event state and runs into the
/// internal API of an animeitor server.
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

/// The stored state of an event, when it already exists.
async fn get_event(
    client: &reqwest::Client,
    event_url: &str,
    internal_token: &str,
) -> Option<EventState> {
    let response = client
        .get(event_url)
        .basic_auth("usuario", Some(internal_token))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let envelope: data::event::Envelope<EventState> = response.json().await.ok()?;
    envelope.data
}

/// True when the event exists (created or updated) after this call.
async fn ensure_event(
    client: &reqwest::Client,
    event_url: &str,
    internal_token: &str,
    state: EventState,
) -> bool {
    // An existing event keeps its salt: PUT replaces every field, and the
    // BOCA shape carries no salt.
    let mut state = state;
    if let Some(existing) = get_event(client, event_url, internal_token).await {
        state.salt = existing.salt;
        match client
            .put(event_url)
            .basic_auth("usuario", Some(internal_token))
            .json(&state)
            .send()
            .await
        {
            Ok(response) => match response.error_for_status() {
                Ok(_) => {
                    debug!("event updated");
                    true
                }
                Err(err) => {
                    error!(?err, "status error updating event");
                    false
                }
            },
            Err(err) => {
                error!(?err, "network error updating event");
                false
            }
        }
    } else {
        match client
            .post(event_url)
            .basic_auth("usuario", Some(internal_token))
            .json(&state)
            .send()
            .await
        {
            Ok(response) => match response.error_for_status() {
                Ok(_) => {
                    debug!("event created");
                    true
                }
                Err(err) => {
                    error!(?err, "status error creating event");
                    false
                }
            },
            Err(err) => {
                error!(?err, "network error creating event");
                false
            }
        }
    }
}

/// Creates the default contest (`""`, empty path segment) if it does not
/// exist. Its empty regex matches every team login, so the public endpoints
/// serve the whole event.
async fn ensure_default_contest(
    client: &reqwest::Client,
    contests_url: &str,
    internal_token: &str,
) {
    let result = client
        .post(contests_url)
        .basic_auth("usuario", Some(internal_token))
        .json(&serde_json::json!({ "name": "", "codes": [""] }))
        .send()
        .await;
    match result {
        Ok(response) => {
            let status = response.status();
            match response.error_for_status() {
                Ok(_) => debug!("default contest created"),
                Err(_) if status == reqwest::StatusCode::CONFLICT => {
                    debug!("default contest already exists")
                }
                Err(err) => error!(?err, "status error creating default contest"),
            }
        }
        Err(err) => error!(?err, "network error creating default contest"),
    }
}

pub async fn db_update_loop(internal_token: &str, boca_url: &str, server_url: &str, event: &str) {
    let dur = tokio::time::Duration::new(1, 0);
    let mut interval = tokio::time::interval(dur);

    let client = reqwest::Client::new();
    let event_url = format!("{server_url}/internal/events/{event}");
    let contests_url = format!("{server_url}/internal/contests/{event}/");
    let runs_url = format!("{event_url}/runs");

    loop {
        interval.tick().await;

        match webcast::load_data_from_url_maybe(boca_url).await {
            Ok(contest_state) => {
                let (state, runs) = from_legacy_contest_state(&contest_state, event);

                if ensure_event(&client, &event_url, internal_token, state).await {
                    ensure_default_contest(&client, &contests_url, internal_token).await;
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
