use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use leptos::leptos_dom::logging::{console_error, console_log};
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

    loop {
        match get_url(&url).await {
            Ok(message) => {
                console_log(&format!("fetched: {url}"));
                return message;
            }
            Err(error) => {
                match error {
                    Error::Gloo(gloo) => console_error(&format!("network error: {gloo:?}")),
                    Error::Serde(serde) => {
                        console_error(&format!("failed to parse response: {serde:?}"))
                    }
                }
                console_log("Wait 5 seconds to reconnect.");
                TimeoutFuture::new(5_000).await;
            }
        }
    }
}
