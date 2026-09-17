use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
enum Error {
    Gloo(gloo_net::Error),
    Serde(serde_json::Error),
}

async fn get_url<M: for<'a> Deserialize<'a>>(url: &str, bearer: Option<&str>) -> Result<M, Error> {
    let mut request = Request::get(url);
    if let Some(key) = bearer {
        request = request.header("Authorization", &format!("Bearer {key}"));
    }
    let resp = request.send().await.map_err(Error::Gloo)?;
    let text = resp.text().await.map_err(Error::Gloo)?;
    let message = serde_json::from_str(&text).map_err(Error::Serde)?;

    Ok(message)
}

pub async fn create_request<M: for<'a> Deserialize<'a> + Serialize + Clone>(url: &str) -> M {
    create_request_maybe_bearer(url, None).await
}

/// Like [`create_request`], sending the site key as a Bearer token (never in
/// the URL, which would leak into access logs).
pub async fn create_request_with_bearer<M: for<'a> Deserialize<'a> + Serialize + Clone>(
    url: &str,
    key: &str,
) -> M {
    create_request_maybe_bearer(url, Some(key)).await
}

async fn create_request_maybe_bearer<M: for<'a> Deserialize<'a> + Serialize + Clone>(
    url: &str,
    bearer: Option<&str>,
) -> M {
    let url = url.to_string();
    info!("create_request: {url}");

    loop {
        match get_url(&url, bearer).await {
            Ok(message) => {
                info!("fetched: {url}");
                return message;
            }
            Err(error) => {
                match error {
                    Error::Gloo(gloo) => error!("network error: {gloo:?}"),
                    Error::Serde(serde) => error!("failed to parse response: {serde:?}"),
                }
                info!("Wait 5 seconds to reconnect.");
                TimeoutFuture::new(5_000).await;
            }
        }
    }
}
