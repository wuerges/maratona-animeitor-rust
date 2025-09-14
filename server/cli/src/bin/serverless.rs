use clap::Parser;

use service::sentry;
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Maratona Rustrimeitor Server
struct Args {
    #[clap(short = 'p', long, default_value = "8000")]
    /// The TCP port to host the server
    port: u16,

    #[clap(short = 'k', long)]
    /// The TCP port to host the server
    api_key: Option<String>,
    // #[clap(short = 'v', long)]
    // /// Maps a local FOLDER to a remote PATH.
    // /// Can be used multiple times.
    // ///
    // /// Expected format: FOLDER:PATH
    // volume: Vec<FromPairArg<Volume>>,
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();
    let _guard = sentry::setup();

    // server_v2::metrics::setup();

    let Args { port, api_key } = Args::parse();

    let args = server_v3::server::Args {
        port,
        server_api_key: api_key,
    };

    // let app_config = AppConfig {
    //     config: complete,
    //     boca_url: url,
    //     server_config,
    //     volumes: volumes.into_iter().map(|x| x.into_inner()).collect(),
    // };

    tracing::info!("\nMaratona Rustreimator rodando!");

    tracing::info!("Server listening on port: {}", port);

    server_v3::server::serve(args).await?;

    Ok(())
}
