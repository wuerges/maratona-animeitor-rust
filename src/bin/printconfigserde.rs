use data::config::*;
use toml;

fn main() -> std::io::Result<()> {

    let contest = contest();

    let result = toml::to_string(&contest).unwrap();

    println!("{}", result);

    Ok(())
}
