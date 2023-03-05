use data::configdata::*;

use rand::{distributions::Alphanumeric, Rng}; // 0.8

fn secret() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(4)
        .map(char::from)
        .collect()
}

fn shuffle_secrets(secrets: &mut ConfigSecret) {
    secrets.salt = Some(secret());

    for sede in &mut secrets.secrets {
        sede.secret = secret();
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    for a in args.iter().skip(1) {
        let f = std::fs::read_to_string(a)?;
        let mut secrets: ConfigSecret = toml::from_str(&f)?;
        shuffle_secrets(&mut secrets);
        let result = toml::to_string(&secrets).unwrap();
        println!("{}", result);
    }

    Ok(())
}
