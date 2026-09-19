//! Management client. Commands compile into a single request; no read-modify-write.
pub mod args;
mod plan;
#[cfg(test)]
mod tests;
use crate::configuration::ServerConfig;
use color_eyre::eyre::{Result, ensure, eyre};
pub use plan::{RequestPlan, plan};
use serde_json::{Value, json};
use std::time::Duration;
use url::Url;

/// Build a URL without interpreting identifiers as path/query syntax.
pub fn request_url(base: &str, request: &RequestPlan) -> Result<Url> {
    let mut url = Url::parse(base)?;
    url.path_segments_mut()
        .map_err(|_| eyre!("server_url must support path segments"))?
        .pop_if_empty()
        .extend(request.segments.iter().map(String::as_str));
    if request.keep_runs {
        url.query_pairs_mut().append_pair("keep_runs", "true");
    }
    Ok(url)
}

pub struct AdminClient {
    client: reqwest::Client,
    base: String,
    username: String,
    token: String,
    timeout: Duration,
}
pub struct Output {
    pub value: Option<Value>,
    pub text: Option<String>,
    pub status: u16,
}
#[derive(Debug)]
pub struct AdminError {
    pub status: Option<u16>,
    pub envelope: Value,
}
impl std::fmt::Display for AdminError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(status) = self.status {
            write!(f, "HTTP {status}: ")?;
        }
        if let Some(errors) = self.envelope["errors"].as_array() {
            for error in errors {
                write!(
                    f,
                    "{}: {} ",
                    error["code"].as_str().unwrap_or("error"),
                    error["message"].as_str().unwrap_or("request failed")
                )?;
            }
        }
        Ok(())
    }
}
impl std::error::Error for AdminError {}
impl AdminError {
    fn new(status: Option<u16>, code: &str, message: &str) -> Self {
        Self {
            status,
            envelope: json!({"errors":[{"code":code,"message":message}]}),
        }
    }
}
impl AdminClient {
    pub fn from_config(server: &ServerConfig) -> Result<Self> {
        let token = server.credential()?;
        Ok(Self {
            client: crate::http_client::build(Some(&server.tls_ca_cert))?,
            base: server.server_url.clone(),
            username: token.name.clone(),
            token: token.token.clone(),
            timeout: Duration::from_secs(30),
        })
    }
    /// Every request has a finite deadline and no automatic mutation retries.
    pub async fn execute(&self, request: &RequestPlan) -> Result<Output, AdminError> {
        let url = request_url(&self.base, request).map_err(|_| {
            AdminError::new(None, "invalid_url", "cannot construct internal API URL")
        })?;
        let mut builder = self
            .client
            .request(request.method.clone(), url)
            .timeout(self.timeout)
            .basic_auth(&self.username, Some(&self.token));
        if let Some(body) = &request.body {
            builder = builder.json(body);
        }
        let response = builder.send().await.map_err(|e| AdminError::new(None, if e.is_timeout() {"timeout"} else {"network_error"}, "request failed; check server_url, credentials, connectivity, and TLS CA configuration"))?;
        let status = response.status();
        if status.as_u16() == 204 {
            return Ok(Output {
                value: None,
                text: None,
                status: 204,
            });
        }
        let text = response.text().await.map_err(|_| {
            AdminError::new(
                Some(status.as_u16()),
                "invalid_response",
                "could not read response",
            )
        })?;
        if !status.is_success() {
            let envelope = serde_json::from_str::<Value>(&text)
                .ok()
                .filter(|v| v["errors"].is_array());
            return Err(match envelope {
                Some(envelope) => AdminError {
                    status: Some(status.as_u16()),
                    envelope,
                },
                None => AdminError::new(
                    Some(status.as_u16()),
                    "http_error",
                    if status.as_u16() == 426 {
                        "internal API requires HTTPS"
                    } else {
                        "server returned a non-JSON error response"
                    },
                ),
            });
        }
        if request.metrics {
            return Ok(Output {
                value: None,
                text: Some(text),
                status: status.as_u16(),
            });
        }
        let mut value: Value = serde_json::from_str(&text).map_err(|_| {
            AdminError::new(
                Some(status.as_u16()),
                "invalid_response",
                "expected a JSON response envelope",
            )
        })?;
        if value.get("data").is_none() || value.get("errors").is_some() {
            return Err(AdminError::new(
                Some(status.as_u16()),
                "invalid_response",
                "successful response must contain data and no errors",
            ));
        }
        if let Some(field) = request.projection {
            let data = value["data"].get(field).cloned().ok_or_else(|| {
                AdminError::new(
                    Some(status.as_u16()),
                    "invalid_response",
                    "event response is missing the requested collection",
                )
            })?;
            value["data"] = data;
        }
        Ok(Output {
            value: Some(value),
            text: None,
            status: status.as_u16(),
        })
    }
}

pub fn render(output: &Output, request: &RequestPlan, json_mode: bool) -> Result<(String, String)> {
    if let Some(text) = &output.text {
        return Ok((
            if json_mode {
                format!("{}\n", serde_json::to_string_pretty(&json!({"data":text}))?)
            } else {
                text.clone()
            },
            String::new(),
        ));
    }
    let Some(value) = &output.value else {
        return Ok((
            if json_mode {
                String::new()
            } else {
                "Deleted.\n".into()
            },
            String::new(),
        ));
    };
    if json_mode {
        return Ok((
            format!("{}\n", serde_json::to_string_pretty(value)?),
            String::new(),
        ));
    }
    let mut warnings = String::new();
    if let Some(entries) = value["warnings"].as_array() {
        for entry in entries {
            warnings.push_str(&format!(
                "Warning {}: {}\n",
                entry["code"].as_str().unwrap_or("unknown"),
                entry["message"].as_str().unwrap_or("")
            ));
        }
    }
    let data = &value["data"];
    let text = if let Some(rows) = data.as_array() {
        if rows.is_empty() {
            "No entries.\n".into()
        } else {
            let mut text = String::new();
            for row in rows {
                if let Some(name) = row.as_str() {
                    text.push_str(&format!("{name}\n"));
                } else if row.get("url").is_some() {
                    text.push_str(&format!(
                        "{}\t{}\t{}\n",
                        row["contest"].as_str().unwrap_or(""),
                        row["site"].as_str().unwrap_or(""),
                        row["url"].as_str().unwrap_or("")
                    ));
                } else {
                    text.push_str(&format!("{}\n", serde_json::to_string(row)?));
                }
            }
            text
        }
    } else if data.get("added").is_some() {
        format!("Added: {}; updated: {}\n", data["added"], data["updated"])
    } else if request.method != reqwest::Method::GET {
        format!(
            "Success (HTTP {}).\n{}\n",
            output.status,
            serde_json::to_string_pretty(data)?
        )
    } else {
        format!("{}\n", serde_json::to_string_pretty(data)?)
    };
    Ok((text, warnings))
}

pub async fn run(args: args::AdminArgs) -> Result<()> {
    let request = plan(args.command)?;
    let server = ServerConfig::load(&args.server_config)?;
    ensure!(
        server.server_url.starts_with("https://"),
        "server_url must use HTTPS"
    );
    let result = AdminClient::from_config(&server)?.execute(&request).await?;
    let (stdout, stderr) = render(&result, &request, args.json)?;
    print!("{stdout}");
    eprint!("{stderr}");
    Ok(())
}
