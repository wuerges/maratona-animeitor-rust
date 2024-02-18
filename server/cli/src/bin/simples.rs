use clap::Parser;
use cli::parse_config;
use color_eyre::Section;
use data::configdata::{ConfigContest, ConfigSecret};
use server::{config::ServerConfig, *};

use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

#[derive(clap::Args)]

struct SimpleArgs {
    #[clap(short = 's', long, default_value = "config/Default.toml")]
    /// Sets a custom config file
    sedes: String,

    #[clap(short = 'x', long)]
    /// Sets a custom config file
    secret: Option<String>,

    #[clap(short = 'p', long, default_value = "8000")]
    /// The TCP port to host the server
    port: u16,

    /// The webcast url from BOCA.
    url: String,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Maratona Rustrimeitor Server
struct SimpleParser {
    #[clap(flatten)]
    args: SimpleArgs,
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    let simple = SimpleParser::parse();

    let SimpleArgs {
        sedes,
        secret,
        port,
        url,
    } = simple.args;

    let config = parse_config::<ConfigContest>(std::path::Path::new(&sedes))
        .map_err(|e| e.with_note(|| "Should be able to parse the config."))?;

    let contest = config.clone().into_contest();

    let config_secret = match secret {
        Some(secret) => parse_config::<ConfigSecret>(std::path::Path::new(&secret))
            .map_err(|e| e.with_note(|| "Should be able to parse secret file."))?,
        None => ConfigSecret::default(),
    }
    .into_secret(&contest);

    let server_config = ServerConfig { port };

    println!("\nSetting up sentry guard");
    let _guard = sentry::setup();
    let _autometrics = metrics::setup();

    tracing::info!("\nMaratona Rustreimator rodando!");
    serve_simple_contest(config, url, config_secret, server_config).await;

    Ok(())
}
