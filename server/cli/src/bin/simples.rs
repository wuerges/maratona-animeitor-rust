use clap::Parser;
use cli::SimpleArgs;

use service::{
    app_config::AppConfig,
    http::{HttpConfig, HttpTlsConfig},
    pair_arg::FromPairArg,
    sentry,
    volume::Volume,
};
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Maratona Rustrimeitor Server
struct SimpleParser {
    #[clap(flatten)]
    args: SimpleArgs,

    #[clap(short = 'p', long, default_value = "8000")]
    /// The TCP port to host the server
    port: u16,

    #[clap(long, requires = "tls_key")]
    /// Path to the TLS certificate chain in PEM format. Enables HTTPS when set together with --tls-key.
    tls_cert: Option<std::path::PathBuf>,

    #[clap(long, requires = "tls_cert")]
    /// Path to the TLS private key in PEM format.
    tls_key: Option<std::path::PathBuf>,

    #[clap(long, default_value = "8443")]
    /// The TCP port for HTTPS. Only used when --tls-cert and --tls-key are set.
    tls_port: u16,

    #[clap(short = 'k')]
    /// API Key for admin endpoints
    server_api_key: Option<String>,

    /// The webcast url from BOCA.
    #[clap(short = 'i')]
    url: Option<String>,

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
        tls_cert,
        tls_key,
        tls_port,
        url,
        volume: volumes,
        server_api_key,
    } = SimpleParser::parse();

    let complete = args.into_contest_and_secret()?;

    let tls = match (tls_cert, tls_key) {
        (Some(cert), Some(key)) => Some(HttpTlsConfig {
            cert,
            key,
            port: tls_port,
        }),
        (None, None) => None,
        _ => unreachable!("clap requires --tls-cert and --tls-key together"),
    };
    let tls_port = tls.as_ref().map(|t| t.port);
    let server_config = HttpConfig { port, tls };

    tracing::info!("\nSetting up sentry guard");
    let _guard = sentry::setup();
    server_v2::metrics::setup();

    let app_config = AppConfig {
        config: complete,
        boca_url: url,
        server_config,
        volumes: volumes.into_iter().map(|x| x.into_inner()).collect(),
        server_api_key,
    };

    tracing::info!("\nMaratona Rustreimator rodando!");

    tracing::info!("Server listening on http://0.0.0.0:{}", port);
    if let Some(port) = tls_port {
        tracing::info!("Server listening on https://0.0.0.0:{}", port);
    }

    server_v2::serve_config(app_config).await?;

    Ok(())
}
