use gloo_timers::future::TimeoutFuture;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct SdkConfig {
    pub api_prefix: String,
    pub ws_prefix: String,
    pub photo_prefix: String,
    pub sound_prefix: String,
    pub photo_url_format: Option<String>,
    pub sound_url_format: Option<String>,
}

#[derive(Deserialize, Default)]
struct ConfigFile {
    api_prefix: Option<String>,
    photo_prefix: Option<String>,
    sound_prefix: Option<String>,
    photo_url_format: Option<String>,
    sound_url_format: Option<String>,
}

fn window_protocol_is_https() -> bool {
    #[cfg(all(target_family = "wasm", not(test)))]
    {
        web_sys::window().is_some_and(|w| {
            w.location()
                .protocol()
                .is_ok_and(|p| p.starts_with("https"))
        })
    }
    #[cfg(any(not(target_family = "wasm"), test))]
    {
        false
    }
}

/// Rewrites absolute `http:` prefixes to `https:` when the page is served
/// over https. Relative prefixes are left untouched (same-origin).
fn upgrade_absolute_http(prefix: String) -> String {
    if window_protocol_is_https() && prefix.starts_with("http:") {
        let mut prefix = prefix;
        prefix.replace_range(.."http:".len(), "https:");
        prefix
    } else {
        prefix
    }
}

fn to_ws_prefix(prefix: &str) -> String {
    let mut prefix = prefix.to_string();
    if prefix.starts_with("https:") {
        prefix.replace_range(.."https:".len(), "wss:");
    }
    if prefix.starts_with("http:") {
        prefix.replace_range(.."http:".len(), "ws:");
    }
    prefix
}

/// Builds an absolute websocket prefix from a relative one using the
/// current page's host (websockets cannot use relative URLs).
fn ws_prefix_from_location(path: &str) -> String {
    #[cfg(all(target_family = "wasm", not(test)))]
    {
        let scheme = if window_protocol_is_https() { "wss" } else { "ws" };
        let host = web_sys::window()
            .and_then(|w| w.location().host().ok())
            .unwrap_or_default();
        format!("{scheme}://{host}{path}")
    }
    #[cfg(any(not(target_family = "wasm"), test))]
    {
        path.to_string()
    }
}

impl SdkConfig {
    /// Same-origin relative defaults, overridden by the legacy compile-time
    /// env vars when set. Runtime `config.json` overrides both (see `load`).
    pub fn from_defaults() -> Self {
        let api_prefix = option_env!("URL_PREFIX")
            .map(String::from)
            .unwrap_or_else(|| "/api".to_string());
        let api_prefix = upgrade_absolute_http(api_prefix);

        SdkConfig {
            ws_prefix: derive_ws_prefix(&api_prefix),
            api_prefix,
            photo_prefix: option_env!("PHOTO_PREFIX")
                .map(String::from)
                .unwrap_or_else(|| "/photos".to_string()),
            sound_prefix: option_env!("SOUND_PREFIX")
                .map(String::from)
                .unwrap_or_else(|| "/sounds".to_string()),
            photo_url_format: option_env!("PHOTO_URL_FORMAT").map(String::from),
            sound_url_format: option_env!("SOUND_URL_FORMAT").map(String::from),
        }
    }

    /// Defaults plus a best-effort same-origin `config.json` fetch.
    pub async fn load() -> Self {
        let mut config = SdkConfig::from_defaults();
        match fetch_config_file().await {
            Ok(file) => config.merge(file),
            Err(err) => log::warn!("could not load config.json: {err}"),
        }
        config
    }

    fn merge(&mut self, file: ConfigFile) {
        if let Some(api_prefix) = file.api_prefix {
            self.api_prefix = upgrade_absolute_http(api_prefix);
            self.ws_prefix = derive_ws_prefix(&self.api_prefix);
        }
        if let Some(photo_prefix) = file.photo_prefix {
            self.photo_prefix = photo_prefix;
        }
        if let Some(sound_prefix) = file.sound_prefix {
            self.sound_prefix = sound_prefix;
        }
        self.photo_url_format = file.photo_url_format.or(self.photo_url_format.take());
        self.sound_url_format = file.sound_url_format.or(self.sound_url_format.take());
    }
}

fn derive_ws_prefix(api_prefix: &str) -> String {
    if api_prefix.starts_with("http") || api_prefix.starts_with("ws") {
        to_ws_prefix(api_prefix)
    } else {
        ws_prefix_from_location(api_prefix)
    }
}

async fn fetch_config_file() -> Result<ConfigFile, String> {
    let request = gloo_net::http::Request::get("config.json").send();
    let timeout = TimeoutFuture::new(2_000);

    let response = match futures::future::select(Box::pin(request), Box::pin(timeout)).await {
        futures::future::Either::Left((result, _)) => result,
        futures::future::Either::Right(_) => return Err("timed out".to_string()),
    };

    let response = response.map_err(|err| err.to_string())?;
    let text = response.text().await.map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_url_prefix() {
        let config = SdkConfig::from_defaults();
        assert_eq!(config.api_prefix, "/api");
    }

    #[test]
    fn check_ws_url_prefix() {
        let config = SdkConfig::from_defaults();
        assert_eq!(config.ws_prefix, "/api");
    }

    #[test]
    fn check_config_file_merge() {
        let mut config = SdkConfig::from_defaults();
        config.merge(ConfigFile {
            api_prefix: Some("https://example.com/api".to_string()),
            photo_prefix: Some("https://cdn.example.com/photos".to_string()),
            ..Default::default()
        });

        assert_eq!(config.api_prefix, "https://example.com/api");
        assert_eq!(config.ws_prefix, "wss://example.com/api");
        assert_eq!(config.photo_prefix, "https://cdn.example.com/photos");
        assert_eq!(config.sound_prefix, "/sounds");
    }
}
