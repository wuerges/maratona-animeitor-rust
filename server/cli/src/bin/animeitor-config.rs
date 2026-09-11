use clap::{Parser, Subcommand};
use std::path::PathBuf;
#[derive(Parser)]
#[command(
    version,
    about = "Generate deployment artifacts from Animeitor configuration"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Compose {
        #[arg(long)]
        event_config: PathBuf,
        #[arg(long)]
        event_secrets: PathBuf,
        #[arg(long)]
        server_config: PathBuf,
        #[arg(long, default_value = ".generated")]
        output_dir: PathBuf,
    },
}
fn main() -> color_eyre::eyre::Result<()> {
    match Args::parse().command {
        Command::Compose {
            event_config,
            event_secrets,
            server_config,
            output_dir,
        } => {
            let value = cli::deployment::compose(
                &event_config,
                &event_secrets,
                &server_config,
                &output_dir,
            )?;
            let path = cli::deployment::write_compose(&value, &output_dir)?;
            println!("{}", path.display());
        }
    }
    Ok(())
}
