use serde::Deserialize;

pub mod test_revelation;

pub fn parse_config<T>(path: &std::path::Path) -> color_eyre::eyre::Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let text = std::fs::read_to_string(path)?;

    let config: T = toml::from_str(&text)?;

    Ok(config)
}
