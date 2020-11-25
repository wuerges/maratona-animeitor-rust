use data::configdata::*;
use toml::Value;
use crate::errors::CResult;

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

pub fn parse_config(path: &std::path::Path) -> std::io::Result<ConfigContest> {
    let text = std::fs::read_to_string(path)?.parse::<Value>()?;

    // let sedes = text.get("sede").unwrap().as_array().map( |s| 
    //     // Sede::new()
    // );

    Ok(ConfigContest::new(vec![]))
}
