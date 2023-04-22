use data::configdata::*;

pub fn pack_contest_config(sedes: ConfigSedes) -> ConfigContest {
    ConfigContest::from_config(sedes.sedes)
}

pub struct ServerConfig<'a> {
    pub port: u16,
    pub photos_path: &'a std::path::Path,
}
