use seed::prelude::*;

pub fn url_prefix() -> &'static str {
    option_env!("URL_PREFIX").unwrap_or_default()
}

pub fn must_remove_ccl() -> bool {
    option_env!("REMOVE_CCL").unwrap_or_default().contains("1")
}

pub fn request(path: &str) -> Request {
    let url = format!("{}/{}", url_prefix(), path);
    // seed::log!("requesting", url);
    Request::from(url)
}

pub async fn fetch_allruns_secret(secret: &str) -> fetch::Result<data::RunsFile> {
    Request::new(format!("{}/allruns_secret?secret={}", url_prefix(), secret))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub async fn fetch_contest() -> fetch::Result<data::ContestFile> {
    let contest: data::ContestFile = request("contest")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await?;

    Ok(match must_remove_ccl() {
        true => contest.remove_ccl(),
        false => contest,
    })
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
    let url = web_sys::Url::new(url_prefix()).expect("Location should be valid");
    url.set_protocol("ws:");
    format!("{}{}", url.href(), path)
}

pub fn photos_prefix() -> &'static str {
    option_env!("PHOTO_PREFIX").unwrap_or_default()
}

pub fn team_photo_location(team_login: &str) -> String {
    std::format!("{}/{}.webp", photos_prefix(), team_login)
}
