use std::path::PathBuf;

pub struct HttpTlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub port: u16,
}

pub struct HttpConfig {
    pub port: u16,
    pub tls: Option<HttpTlsConfig>,
}
