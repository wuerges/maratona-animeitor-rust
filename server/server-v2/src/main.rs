use clap::Parser;
use cli::{pair_arg::FromPairArg, sentry};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

use service::{
    app_config::AppConfig,
    http::{HttpConfig, HttpTlsConfig},
    volume::Volume,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Maratona Rustrimeitor Server
struct SimpleParser {
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

    #[clap(long)]
    /// TOML file containing named credentials for the internal API.
    internal_tokens: PathBuf,

    #[clap(short = 'v', long)]
    /// Maps a local FOLDER to a remote PATH.
    /// Can be used multiple times.
    ///
    /// Expected format: FOLDER:PATH
    volume: Vec<FromPairArg<Volume>>,
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing::info!("\nSetting up sentry guard");
    let _guard = sentry::setup();

    let SimpleParser {
        port,
        tls_cert,
        tls_key,
        tls_port,
        volume: volumes,
        internal_tokens,
    } = SimpleParser::parse();

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
    #[derive(Deserialize)]
    struct TokenFile {
        tokens: Vec<TokenEntry>,
    }
    #[derive(Deserialize)]
    struct TokenEntry {
        name: String,
        token: String,
        #[serde(default = "enabled")]
        enabled: bool,
    }
    fn enabled() -> bool {
        true
    }
    let raw = std::fs::read_to_string(&internal_tokens)?;
    let file: TokenFile = toml::from_str(&raw)?;
    let mut tokens = HashMap::new();
    for entry in file.tokens.into_iter().filter(|t| t.enabled) {
        if entry.name.is_empty() || entry.token.is_empty() {
            color_eyre::eyre::bail!("enabled internal tokens need a non-empty name and token");
        }
        if tokens.insert(entry.name.clone(), entry.token).is_some() {
            color_eyre::eyre::bail!("duplicate internal token name: {}", entry.name);
        }
    }
    if tokens.is_empty() {
        color_eyre::eyre::bail!("internal token file contains no enabled tokens");
    }
    let server_config = HttpConfig { port, tls };

    server_v2::metrics::setup();

    let app_config = AppConfig {
        server_config,
        volumes: volumes.into_iter().map(|x| x.into_inner()).collect(),
        internal_tokens: tokens,
    };

    tracing::info!("\nMaratona Rustreimator rodando!");

    tracing::info!("Server listening on http://0.0.0.0:{}", port);
    if let Some(port) = tls_port {
        tracing::info!("Server listening on https://0.0.0.0:{}", port);
    }

    server_v2::serve_config(app_config).await?;

    Ok(())
}
