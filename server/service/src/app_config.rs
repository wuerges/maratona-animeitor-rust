use std::collections::HashMap;

use data::configdata::{ConfigContest, Contest, Secret};

use crate::{http::HttpConfig, volume::Volume};

pub struct AppConfig {
    pub config: HashMap<String, (ConfigContest, Contest, Secret)>,
    pub boca_url: String,
    pub server_config: HttpConfig,
    pub volumes: Vec<Volume>,
    pub server_api_key: Option<String>,
}
