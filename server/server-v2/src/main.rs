use clap::Parser;
use cli::{configuration::ServerConfig, sentry};
use service::{
    app_config::AppConfig,
    http::{HttpConfig, HttpTlsConfig},
    volume::Volume,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about = "Event-independent Animeitor HTTP/HTTPS server")]
struct Args {
    #[arg(long)]
    server_config: PathBuf,
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    let _guard = sentry::setup();
    let result = run(Args::parse()).await;
    if let Err(error) = &result {
        sentry::report_failure(error.as_ref());
    }
    result
}

async fn run(args: Args) -> color_eyre::eyre::Result<()> {
    let config = ServerConfig::load(&args.server_config)?;
    let tokens = config
        .tokens
        .into_iter()
        .filter(|t| t.enabled)
        .map(|t| (t.name, t.token))
        .collect();
    server_v2::metrics::setup();
    server_v2::serve_config(AppConfig {
        database: config.database,
        public_url: config.public_url.parse()?,
        revelation_salt: config.revelation_salt,
        server_config: HttpConfig {
            port: config.public_port,
            tls: Some(HttpTlsConfig {
                cert: config.tls_cert,
                key: config.tls_key,
                port: config.tls_port,
            }),
        },
        volumes: config
            .assets
            .into_iter()
            .map(|a| Volume {
                folder: a.directory.to_string_lossy().into_owned(),
                path: a.path,
            })
            .collect(),
        internal_tokens: tokens,
    })
    .await?;
    Ok(())
}
