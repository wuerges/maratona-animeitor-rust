use clap::Parser;
use cli::SimpleArgs;
use server::{config::ServerConfig, *};

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
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    let SimpleParser { args, port, url } = SimpleParser::parse();

    let (config_contest, _, config_secret) = args.into_contest_and_secret()?;

    let server_config = ServerConfig { port };

    println!("\nSetting up sentry guard");
    let _guard = sentry::setup();
    let _autometrics = metrics::setup();

    tracing::info!("\nMaratona Rustreimator rodando!");
    serve_simple_contest(config_contest, url, config_secret, server_config).await;

    Ok(())
}
