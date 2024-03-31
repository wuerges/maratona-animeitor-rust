use std::str::FromStr;

use clap::Parser;
use cli::SimpleArgs;
use color_eyre::eyre::eyre;
use server::*;

use service::{
    app_config::AppConfig, http::HttpConfig, pair_arg::FromPairArg, sentry, volume::Volume,
};
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

    #[clap(long)]
    server_version: Option<Version>,
}

#[derive(Default, Debug, Clone)]
enum Version {
    #[default]
    V1,
    V2,
}

impl FromStr for Version {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v1" => Ok(Version::V1),
            "v2" => Ok(Version::V2),
            _ => Err(eyre!("unknown version: {s}")),
        }
    }
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
        server_version,
    } = SimpleParser::parse();

    let complete = args.into_contest_and_secret()?;

    let server_config = HttpConfig { port };

    tracing::info!("\nSetting up sentry guard");
    let _guard = sentry::setup();
    let _autometrics = metrics::setup();

    let app_config = AppConfig {
        config: complete,
        boca_url: url,
        server_config,
        volumes: volumes.into_iter().map(|x| x.into_inner()).collect(),
    };

    tracing::info!("\nMaratona Rustreimator rodando!");

    tracing::info!("Server listening on port: {}", port);

    match server_version.unwrap_or_default() {
        Version::V1 => serve_simple_contest(app_config).await,
        Version::V2 => server_v2::serve_config(app_config).await?,
    }

    Ok(())
}
