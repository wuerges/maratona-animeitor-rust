use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use leptos::logging::*;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
enum Error {
    Gloo(gloo_net::Error),
    Serde(serde_json::Error),
}

async fn get_url<M: for<'a> Deserialize<'a>>(url: &str) -> Result<M, Error> {
    let resp = Request::get(url).send().await.map_err(Error::Gloo)?;
    let text = resp.text().await.map_err(Error::Gloo)?;
    let message = serde_json::from_str(&text).map_err(Error::Serde)?;

    Ok(message)
}

pub async fn create_request<M: for<'a> Deserialize<'a> + Serialize + Clone>(url: &str) -> M {
    let url = url.to_string();
    log!("create_request: {url}");

    loop {
        match get_url(&url).await {
            Ok(message) => {
                log!("fetched: {url}");
                return message;
            }
            Err(error) => {
                match error {
                    Error::Gloo(gloo) => error!("network error: {gloo:?}"),
                    Error::Serde(serde) => {
                        error!("failed to parse response: {serde:?}")
                    }
                }
                log!("Wait 5 seconds to reconnect.");
                TimeoutFuture::new(5_000).await;
            }
        }
    }
}
