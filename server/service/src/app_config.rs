use crate::{http::HttpConfig, volume::Volume};

pub struct AppConfig {
    pub revelation_salt: String,
    pub server_config: HttpConfig,
    pub volumes: Vec<Volume>,
    pub internal_tokens: std::collections::HashMap<String, String>,
}
