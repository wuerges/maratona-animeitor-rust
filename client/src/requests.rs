use seed::prelude::*;

pub fn url_prefix() -> &'static str {
    option_env!("URL_PREFIX").unwrap_or("")
}

pub fn request(path: &str) -> Request {
    let url = format!("{}/{}", url_prefix(), path);
    // seed::log!("requesting", url);
    Request::from(url)
}

pub async fn fetch_allruns_secret(secret: &String) -> fetch::Result<data::RunsFile> {
    Request::new(format!("{}/allruns_secret?secret={}", url_prefix(), secret))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub async fn fetch_contest() -> fetch::Result<data::ContestFile> {
    request("contest")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub async fn fetch_config() -> fetch::Result<data::configdata::ConfigContest> {
    request("config")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub fn get_ws_url(path: &str) -> String {
    let base_url = option_env!("URL_PREFIX").map_or(
        {
            web_sys::window()
                .expect("Should have a window")
                .location()
                .href()
                .expect("Should have a URL")
        },
        |v| v.to_string(),
    );

    let url = web_sys::Url::new(&base_url).expect("Location should be valid");
    url.set_protocol("ws:");
    url.set_pathname(path);
    url.href()
}
