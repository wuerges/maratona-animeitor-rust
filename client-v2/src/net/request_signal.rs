use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use leptos::{
    leptos_dom::logging::{console_error, console_log},
    *,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

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

pub fn create_request_signal<M: for<'a> Deserialize<'a> + Serialize + Clone>(
    url: &str,
    initial: M,
) -> ReadSignal<M> {
    let (message, set_message) = create_signal(initial);
    let url = url.to_string();

    spawn_local(async move {
        loop {
            match get_url(&url).await {
                Ok(message) => {
                    console_log(&format!("fetched: {url}"));
                    set_message.set(message);
                    break;
                }
                Err(error) => {
                    console_error(&format!("failed to load contest: {error:?}"));
                    console_log("Wait 5 seconds to reconnect.");
                    TimeoutFuture::new(5_000).await;
                }
            }
        }
    });

    message
}
