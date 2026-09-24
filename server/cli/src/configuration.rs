//! The three user-maintained configuration files. Never log private structures.
use color_eyre::eyre::{Context, Result, ensure, eyre};
use data::event::{ContestConfig, SiteConfig};
use serde::{Deserialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

pub fn read<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let raw =
        std::fs::read_to_string(path).wrap_err_with(|| format!("reading {}", path.display()))?;
    // TOML's Display includes source lines, which may contain credentials.
    toml::from_str(&raw).map_err(|_| {
        eyre!(
            "invalid configuration in {} (check field names, types, and duplicate entries)",
            path.display()
        )
    })
}
pub fn absolute(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}
pub fn relative(file: &Path, path: &Path) -> Result<PathBuf> {
    absolute(&file.parent().unwrap_or(Path::new(".")).join(path))
}
fn nonempty(value: &str, field: &str) -> Result<()> {
    ensure!(!value.trim().is_empty(), "{field} must not be empty");
    Ok(())
}
fn unique<'a>(names: impl Iterator<Item = &'a str>, field: &str) -> Result<()> {
    let mut seen = HashSet::new();
    for name in names {
        nonempty(name, field)?;
        ensure!(seen.insert(name), "duplicate {field}");
    }
    Ok(())
}
fn one() -> usize {
    1
}
fn two() -> usize {
    2
}
fn three() -> usize {
    3
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventConfig {
    pub event: Event,
    pub contests: Vec<Contest>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub name: String,
    pub secret: String,
    pub score_freeze_time_seconds: Option<i64>,
    pub photo_url_format: Option<String>,
    pub sound_url_format: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contest {
    pub name: String,
    pub codes: Vec<String>,
    #[serde(default)]
    pub secret: String,
    pub style: Option<String>,
    #[serde(default = "one")]
    pub ouro: usize,
    #[serde(default = "two")]
    pub prata: usize,
    #[serde(default = "three")]
    pub bronze: usize,
    #[serde(default)]
    pub sites: Vec<Site>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    pub name: String,
    pub codes: Vec<String>,
    #[serde(default)]
    pub secret: String,
    // Preserve legacy display metadata even though the site API uses name/codes.
    pub style: Option<String>,
    pub ouro: Option<usize>,
    pub prata: Option<usize>,
    pub bronze: Option<usize>,
}
pub struct ConfiguredContest {
    pub config: ContestConfig,
    pub sites: Vec<SiteConfig>,
}
impl EventConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let value: Self = read(path)?;
        value
            .validate()
            .wrap_err_with(|| format!("validating {}", path.display()))?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<()> {
        nonempty(&self.event.name, "event.name")?;
        nonempty(&self.event.secret, "event.secret")?;
        ensure!(!self.contests.is_empty(), "contests must not be empty");
        unique(
            self.contests.iter().map(|c| c.name.as_str()),
            "contest name",
        )?;
        for c in &self.contests {
            regex::RegexSet::new(&c.codes).map_err(|_| eyre!("invalid contest codes"))?;
            unique(
                c.sites.iter().map(|s| s.name.as_str()),
                "site name within contest",
            )?;
            for s in &c.sites {
                regex::RegexSet::new(&s.codes).map_err(|_| eyre!("invalid site codes"))?;
            }
        }
        Ok(())
    }
    pub fn configured(&self) -> Vec<ConfiguredContest> {
        self.contests
            .iter()
            .map(|c| ConfiguredContest {
                config: ContestConfig {
                    name: c.name.clone(),
                    codes: c.codes.clone(),
                    salt: Some(c.secret.clone()),
                    style: c.style.clone(),
                    ouro: c.ouro,
                    prata: c.prata,
                    bronze: c.bronze,
                },
                sites: c
                    .sites
                    .iter()
                    .map(|s| SiteConfig {
                        name: s.name.clone(),
                        codes: s.codes.clone(),
                        salt: Some(s.secret.clone()),
                    })
                    .collect(),
            })
            .collect()
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventSecrets {
    pub webcasts: BTreeMap<String, String>,
}
impl EventSecrets {
    pub fn source(path: &Path, event: &str) -> Result<String> {
        let config: Self = read(path)?;
        for (name, source) in &config.webcasts {
            nonempty(name, "webcast event name")?;
            nonempty(source, "webcast source")?;
        }
        let source = config
            .webcasts
            .get(event)
            .ok_or_else(|| eyre!("{} has no webcast for selected event", path.display()))?;
        if source.starts_with("https://") || source.starts_with("http://") {
            url::Url::parse(source)
                .map_err(|_| eyre!("invalid webcast URL in {}", path.display()))?;
            Ok(source.clone())
        } else {
            Ok(relative(path, Path::new(source))?
                .to_string_lossy()
                .into_owned())
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    #[serde(default)]
    pub database: service::database::DatabaseConfig,
    pub revelation_salt: String,
    pub public_port: u16,
    pub tls_port: u16,
    pub tls_cert: PathBuf,
    pub tls_key: PathBuf,
    pub tls_ca_cert: PathBuf,
    pub server_url: String,
    pub public_url: String,
    pub client_token: String,
    pub tokens: Vec<Token>,
    #[serde(default)]
    pub assets: Vec<Asset>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Token {
    pub name: String,
    pub token: String,
    #[serde(default = "enabled")]
    pub enabled: bool,
}
fn enabled() -> bool {
    true
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub directory: PathBuf,
    pub path: String,
}
impl ServerConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let mut value: Self = read(path)?;
        value
            .validate()
            .wrap_err_with(|| format!("validating {}", path.display()))?;
        if let service::database::DatabaseConfig::Sqlite {
            path: database_path,
        } = &mut value.database
        {
            *database_path = relative(path, database_path)?;
        }
        value.tls_cert = relative(path, &value.tls_cert)?;
        value.tls_key = relative(path, &value.tls_key)?;
        value.tls_ca_cert = relative(path, &value.tls_ca_cert)?;
        for a in &mut value.assets {
            a.directory = relative(path, &a.directory)?;
        }
        Ok(value)
    }
    pub fn validate(&self) -> Result<()> {
        if let service::database::DatabaseConfig::Sqlite { path } = &self.database {
            ensure!(
                !path.as_os_str().is_empty(),
                "database.path must not be empty"
            );
            ensure!(
                path != Path::new(":memory:"),
                "use database.type = memory for an ephemeral database"
            );
        }
        nonempty(&self.revelation_salt, "revelation_salt")?;
        ensure!(
            self.public_port > 0 && self.tls_port > 0 && self.public_port != self.tls_port,
            "ports must be nonzero and distinct"
        );
        for (raw, field, https) in [
            (&self.server_url, "server_url", true),
            (&self.public_url, "public_url", false),
        ] {
            let u = url::Url::parse(raw).map_err(|_| eyre!("invalid {field}"))?;
            ensure!(
                u.host_str().is_some()
                    && (u.scheme() == "https" || (!https && u.scheme() == "http")),
                "invalid scheme or host in {field}"
            );
            ensure!(
                u.username().is_empty()
                    && u.password().is_none()
                    && u.query().is_none()
                    && u.fragment().is_none(),
                "{field} must not contain credentials, query, or fragment"
            );
        }
        for p in [&self.tls_cert, &self.tls_key, &self.tls_ca_cert] {
            ensure!(!p.as_os_str().is_empty(), "TLS paths must not be empty");
        }
        unique(self.tokens.iter().map(|t| t.name.as_str()), "token name")?;
        for t in self.tokens.iter().filter(|t| t.enabled) {
            nonempty(&t.token, "token")?;
        }
        self.credential()?;
        Ok(())
    }
    pub fn credential(&self) -> Result<&Token> {
        self.tokens
            .iter()
            .find(|t| t.enabled && t.name == self.client_token)
            .ok_or_else(|| eyre!("client_token must select an enabled token"))
    }
}
