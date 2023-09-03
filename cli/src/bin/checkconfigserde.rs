use data::configdata::*;

fn main() -> eyre::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    for a in args.iter().skip(1) {
        println!("args: {}", a);
        let f = std::fs::read_to_string(a)?;
        let contest: ConfigContest = toml::from_str(&f)?;
        let result = toml::to_string(&contest).unwrap();
        println!("{}", result);
    }

    Ok(())
}
