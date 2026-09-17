use std::path::Path;

use color_eyre::eyre::{Context, Result};

/// Trust an optional PEM CA bundle in addition to the system roots.
pub fn build(ca_cert: Option<&Path>) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder();
    if let Some(path) = ca_cert {
        let pem = std::fs::read(path)
            .wrap_err_with(|| format!("reading TLS CA certificate {}", path.display()))?;
        let certs = reqwest::Certificate::from_pem_bundle(&pem)?;
        color_eyre::eyre::ensure!(!certs.is_empty(), "TLS CA file contains no certificates");
        for cert in certs {
            builder = builder.add_root_certificate(cert);
        }
    }
    Ok(builder.build()?)
}
