use clap::Parser;
use cli::SimpleArgs;
use server::{config::ServerConfig, *};

use service::{pair_arg::FromPairArg, volume::Volume};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Maratona Rustrimeitor Server
struct SimpleParser {
    #[clap(flatten)]
    args: SimpleArgs,

    #[clap(short = 'p', long, default_value = "8000")]
    /// The TCP port to host the server
    port: u16,

    /// The webcast url from BOCA.
    url: String,

    #[clap(short = 'v', long)]
    /// Maps a local FOLDER to a remote PATH.
    /// Can be used multiple times.
    ///
    /// Expected format: FOLDER:PATH
    volume: Vec<FromPairArg<Volume>>,
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    let SimpleParser {
        args,
        port,
        url,
        volume: volumes,
    } = SimpleParser::parse();

    let (config_contest, _, config_secret) = args.into_contest_and_secret()?;

    let server_config = ServerConfig { port };

    tracing::info!("\nSetting up sentry guard");
    let _guard = sentry::setup();
    let _autometrics = metrics::setup();

    tracing::info!("\nMaratona Rustreimator rodando!");
    serve_simple_contest(
        [("test".to_string(), config_contest)].into_iter().collect(),
        url,
        config_secret,
        server_config,
        volumes.into_iter().map(|x| x.into_inner()).collect(),
    )
    .await;

    Ok(())
}
