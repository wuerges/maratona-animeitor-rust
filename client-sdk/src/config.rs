const DEFAULT_URL: &str = "http://0.0.0.0";

#[derive(Debug, Clone)]
pub struct SdkConfig {
    pub api_prefix: String,
    pub ws_prefix: String,
    pub photo_prefix: String,
    pub sound_prefix: String,
    pub photo_url_format: Option<String>,
    pub sound_url_format: Option<String>,
}

fn window_origin() -> Option<String> {
    #[cfg(all(target_family = "wasm", not(test)))]
    {
        web_sys::window().map(|w| w.origin())
    }
    #[cfg(any(not(target_family = "wasm"), test))]
    {
        None
    }
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

fn guess_prefix() -> String {
    match window_origin() {
        Some(origin) => origin,
        None => {
            log::warn!("could not guess an origin, using default: {}", DEFAULT_URL);
            DEFAULT_URL.to_string()
        }
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

impl SdkConfig {
    pub fn from_env() -> Self {
        let api_prefix = match option_env!("URL_PREFIX") {
            Some(prefix) => prefix.to_string(),
            None => format!("{}/api", guess_prefix()),
        };
        let api_prefix = if window_protocol_is_https() && api_prefix.starts_with("http:") {
            let mut prefix = api_prefix;
            prefix.replace_range(.."http:".len(), "https:");
            prefix
        } else {
            api_prefix
        };
        let ws_prefix = to_ws_prefix(&api_prefix);

        let photo_prefix = match option_env!("PHOTO_PREFIX") {
            Some(prefix) => prefix.to_string(),
            None => format!("{}/photos", guess_prefix()),
        };
        let sound_prefix = match option_env!("SOUND_PREFIX") {
            Some(prefix) => prefix.to_string(),
            None => format!("{}/sounds", guess_prefix()),
        };

        SdkConfig {
            api_prefix,
            ws_prefix,
            photo_prefix,
            sound_prefix,
            photo_url_format: option_env!("PHOTO_URL_FORMAT").map(String::from),
            sound_url_format: option_env!("SOUND_URL_FORMAT").map(String::from),
        }
    }
}
