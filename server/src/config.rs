use data::configdata::*;

// fn sede_from_value(v : Value) -> CResult<Sede> {
//     let nome = v.get("nome").unwrap();
//     let codigo = v.get("nome").unwrap();
//     let premiacao = v.get("nome").unwrap().as_bool().unwrap();
//     let vagas = v.get("nome").unwrap().as_integer().unwrap();
//     Ok(Sede::new(
//         nome,
//         "",
//         "",
//         codigo,
//         premiacao,
//         vagas,
//     ))
// }

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
