use std::path::Path;

use clap::Parser;
use cli::test_revelation;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Golden model generator
struct Args {
    #[clap(required)]
    webcast: Path,
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    let args = Args::parse();

    for result in test_revelation::build_revelation(args.webcast).await? {
        println!("{}", result);
    }

    Ok(())
}
