use crate::{http::HttpConfig, volume::Volume};

pub struct AppConfig {
    pub boca_url: Option<String>,
    pub server_config: HttpConfig,
    pub volumes: Vec<Volume>,
    pub internal_token: Option<String>,
    /// The event fed by the in-process BOCA loop (`-i`).
    pub default_event: String,
}
