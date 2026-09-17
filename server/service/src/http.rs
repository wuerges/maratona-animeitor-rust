use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::errors::ServiceResult;

pub struct HttpTlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub port: u16,
}

pub struct HttpConfig {
    pub port: u16,
    pub tls: Option<HttpTlsConfig>,
}

/// Loads a rustls server config from PEM files.
pub fn load_rustls_config(cert: &Path, key: &Path) -> ServiceResult<rustls::ServerConfig> {
    let certs = rustls_pemfile::certs(&mut BufReader::new(File::open(cert)?))
        .collect::<std::io::Result<Vec<_>>>()?;
    let key = rustls_pemfile::private_key(&mut BufReader::new(File::open(key)?))?
        .ok_or_else(|| std::io::Error::other("no private key found in PEM file"))?;
    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(std::io::Error::other)
        .map_err(Into::into)
}
