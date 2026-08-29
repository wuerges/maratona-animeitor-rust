use crate::{http::HttpConfig, volume::Volume};

pub struct AppConfig {
    pub server_config: HttpConfig,
    pub volumes: Vec<Volume>,
    pub internal_token: Option<String>,
}
