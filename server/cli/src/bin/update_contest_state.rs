use clap::Parser;

use color_eyre::eyre;
use service::{sentry, webcast};
use tracing::{debug, error};
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Maratona Rustrimeitor Server
struct SimpleParser {
    #[clap(short = 'k')]
    /// API Key for admin endpoints
    server_api_key: String,

    /// The webcast url from BOCA.
    #[clap(short = 'i')]
    boca_url: String,

    /// The webcast url from BOCA.
    #[clap(short = 's')]
    server_url: String,
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    let SimpleParser {
        server_api_key,
        boca_url,
        server_url,
    } = SimpleParser::parse();

    tracing::info!("\nSetting up sentry guard");
    let _guard = sentry::setup();

    db_update_loop(&server_api_key, &boca_url, &server_url).await?;

    Ok(())
}

#[allow(clippy::type_complexity)]
pub async fn db_update_loop(
    server_api_key: &str,
    boca_url: &str,
    server_url: &str,
) -> eyre::Result<()> {
    let dur = tokio::time::Duration::new(1, 0);
    let mut interval = tokio::time::interval(dur);

    let client = reqwest::Client::new();

    loop {
        interval.tick().await;

        let contest_state = webcast::load_data_from_url_maybe(&boca_url).await?;

        let result = client
            .put(format!("{server_url}/contests"))
            .json(&contest_state)
            .header("apikey", server_api_key)
            .send()
            .await?;

        match result.error_for_status() {
            Ok(_) => {
                debug!("ok");
            }
            Err(err) => error!(?err, "status error"),
        }
    }
}
