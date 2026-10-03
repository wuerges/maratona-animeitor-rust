//! Validated credentials. Do not log or serialize these records.
use data::internal_auth::{InternalRole, TokenCapabilities};
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(try_from = "RawToken")]
pub struct InternalToken {
    pub name: String,
    pub token: String,
    pub enabled: bool,
    pub role: InternalRole,
    events: Vec<String>,
    matchers: Vec<Regex>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawToken {
    name: String,
    token: String,
    #[serde(default = "enabled")]
    enabled: bool,
    role: InternalRole,
    events: Vec<String>,
}
fn enabled() -> bool {
    true
}

impl TryFrom<RawToken> for InternalToken {
    type Error = &'static str;
    fn try_from(raw: RawToken) -> Result<Self, Self::Error> {
        let mut value = Self::new(raw.name, raw.token, raw.role, raw.events)?;
        value.enabled = raw.enabled;
        Ok(value)
    }
}

impl InternalToken {
    pub fn new(
        name: String,
        token: String,
        role: InternalRole,
        events: Vec<String>,
    ) -> Result<Self, &'static str> {
        if name.trim().is_empty() || name.contains(':') || token.trim().is_empty() {
            return Err("credential name or token is invalid");
        }
        let mut seen = std::collections::HashSet::new();
        let mut matchers = Vec::new();
        for pattern in &events {
            if pattern.trim().is_empty() || !seen.insert(pattern) {
                return Err("blank or duplicate event pattern");
            }
            matchers.push(
                Regex::new(&format!(r"\A(?:{pattern})\z")).map_err(|_| "invalid event pattern")?,
            );
        }
        Ok(Self {
            name,
            token,
            enabled: true,
            role,
            events,
            matchers,
        })
    }
    pub fn owns(&self, event: &str) -> bool {
        self.matchers.iter().any(|pattern| pattern.is_match(event))
    }
    pub fn capabilities(&self) -> TokenCapabilities {
        TokenCapabilities {
            name: self.name.clone(),
            role: self.role,
            events: self.events.clone(),
        }
    }
}
