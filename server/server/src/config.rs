use data::configdata::*;

pub fn pack_contest_config(sedes: ConfigSedes) -> ConfigContest {
    ConfigContest::from_config(sedes.sedes)
}

pub struct ServerConfig {
    pub port: u16,
}
