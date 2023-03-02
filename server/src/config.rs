use data::configdata::*;
use serde::Deserialize;

pub fn pack_contest_config(sedes: ConfigSedes) -> ConfigContest {
    ConfigContest::from_config(sedes.sedes)
}

pub fn parse_config<T>(path: &std::path::Path) -> std::io::Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let text = std::fs::read_to_string(path)?;

    let config: T = toml::from_str(&text)?;

    Ok(config)
}

pub struct ServerConfig<'a> {
    pub port: u16,
    pub photos_path: &'a std::path::Path,
}
