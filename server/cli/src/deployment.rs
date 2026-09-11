//! Derived deployment artifacts; no credentials are put in Compose arguments.
use crate::configuration::{EventConfig, EventSecrets, ServerConfig, absolute};
use color_eyre::eyre::{Result, ensure};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
};

fn bind(source: &Path, target: &Path) -> Value {
    json!({"type":"bind","source":source,"target":target,"read_only":true,"bind":{"create_host_path":false}})
}
fn mounts(paths: &[&Path]) -> Vec<Value> {
    let mut seen = BTreeMap::new();
    for path in paths {
        seen.insert(path.to_path_buf(), bind(path, path));
    }
    seen.into_values().collect()
}
fn private_write(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    file.write_all(bytes)?;
    Ok(())
}
pub fn compose(
    event_path: &Path,
    secrets_path: &Path,
    server_path: &Path,
    output: &Path,
) -> Result<Value> {
    let event_path = absolute(event_path)?;
    let secrets_path = absolute(secrets_path)?;
    let server_path = absolute(server_path)?;
    let event = EventConfig::load(&event_path)?;
    let source = EventSecrets::source(&secrets_path, &event.event.name)?;
    let server = ServerConfig::load(&server_path)?;
    let mut server_mounts = mounts(&[&server_path, &server.tls_cert, &server.tls_key]);
    for m in &server.docker.mounts {
        ensure!(
            m.source != event_path
                && m.source != secrets_path
                && !event_path.starts_with(&m.source)
                && !secrets_path.starts_with(&m.source),
            "server Docker mounts must not expose event configuration files"
        );
        server_mounts.push(bind(&m.source, &m.target));
    }
    let mut feeder_mounts = mounts(&[
        &server_path,
        &event_path,
        &secrets_path,
        &server.tls_ca_cert,
    ]);
    if !source.starts_with("http://") && !source.starts_with("https://") {
        feeder_mounts.push(bind(Path::new(&source), Path::new(&source)));
    }
    let output = absolute(output)?;
    std::fs::create_dir_all(&output)?;
    let prometheus_config = output.join("prometheus.json");
    let token_file = output.join("prometheus-token");
    let url = url::Url::parse(&server.server_url)?;
    let host = url.host().expect("validated host").to_string();
    let target = format!("{}:{}", host, url.port_or_known_default().unwrap_or(443));
    let token = server.credential()?;
    private_write(&token_file, token.token.as_bytes())?;
    let metrics = json!({"global":{"scrape_interval":"15s"},"scrape_configs":[{"job_name":"animeitor","scheme":"https","metrics_path":format!("{}/internal/metrics",url.path().trim_end_matches('/')),"tls_config":{"ca_file":"/etc/animeitor-ca.pem"},"basic_auth":{"username":token.name,"password_file":"/etc/animeitor-token"},"static_configs":[{"targets":[target]}]}]});
    private_write(&prometheus_config, &serde_json::to_vec_pretty(&metrics)?)?;
    let image = "wuerges/animeitor:latest";
    Ok(json!({"name":"animeitor","services":{
        "animeitor":{"image":image,"entrypoint":["/animeitor-server"],"command":["--server-config",server_path],"volumes":server_mounts,"ports":[format!("{}:{}",server.public_port,server.public_port),format!("{}:{}",server.tls_port,server.tls_port)],"ulimits":{"nofile":{"soft":65536,"hard":65536}},"environment":{"RUST_LOG":"info"}},
        "feeder":{"image":image,"entrypoint":["/animeitor-feeder"],"command":["--event-config",event_path,"--event-secrets",secrets_path,"--server-config",server_path],"volumes":feeder_mounts,"depends_on":["animeitor"],"environment":{"RUST_LOG":"info"}},
        "printurls":{"image":image,"entrypoint":["/printurls"],"command":["--event-config",event_path,"--server-config",server_path],"volumes":mounts(&[&server_path,&event_path])},
        "prometheus":{"image":"prom/prometheus:latest","profiles":["monitoring"],"user":"0:0","ports":["9090:9090"],"command":["--config.file=/etc/animeitor-prometheus.json","--storage.tsdb.path=/prometheus"],"volumes":[bind(&prometheus_config,Path::new("/etc/animeitor-prometheus.json")),bind(&token_file,Path::new("/etc/animeitor-token")),bind(&server.tls_ca_cert,Path::new("/etc/animeitor-ca.pem")),{"type":"volume","source":"prometheus-data","target":"/prometheus"}],"depends_on":["animeitor"]}
    },"volumes":{"prometheus-data":{}}}))
}
pub fn write_compose(value: &Value, output: &Path) -> Result<PathBuf> {
    let path = output.join("compose.json");
    private_write(&path, &serde_json::to_vec_pretty(value)?)?;
    Ok(path)
}
