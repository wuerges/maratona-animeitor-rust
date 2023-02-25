use cli::test_revelation;

extern crate clap;
use clap::{App, Arg};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let matches = App::new("Golden model generator")
        .arg(Arg::with_name("WEBCAST").required(true))
        .get_matches();
    let input_file = matches
        .value_of("WEBCAST")
        .expect("Expected webcast parameter");
    for result in test_revelation::build_revelation(input_file).await? {
        println!("{}", result);
    }
    Ok(())
}
