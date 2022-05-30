use data::configdata::*;

pub fn pack_contest_config(
    sedes: ConfigSedes,
    escolas: ConfigEscolas,
    teams: ConfigTeams,
) -> ConfigContest {
    ConfigContest::from_config(sedes.sedes, escolas.escolas, teams.teams)
}

pub fn parse_config_sedes(path: &std::path::Path) -> std::io::Result<ConfigSedes> {
    let text = std::fs::read_to_string(path)?;

    let config: ConfigSedes = toml::from_str(&text)?;

    Ok(config)
}
pub fn parse_config_escolas(path: &std::path::Path) -> std::io::Result<ConfigEscolas> {
    let text = std::fs::read_to_string(path)?;

    let config: ConfigEscolas = toml::from_str(&text)?;

    Ok(config)
}
pub fn parse_config_teams(path: &std::path::Path) -> std::io::Result<ConfigTeams> {
    let text = std::fs::read_to_string(path)?;

    let config: ConfigTeams = toml::from_str(&text)?;

    Ok(config)
}
