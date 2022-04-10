use seed::prelude::*;
use std::env::*;
use data;

pub fn url_prefix() -> &'static str {
  option_env!("URL_PREFIX").unwrap_or("")
}

pub async fn fetch_allruns_secret(secret : &String) -> fetch::Result<data::RunsFile> {
    Request::new(format!("{}/allruns_{}", url_prefix(), secret))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}


pub async fn fetch_contest() -> fetch::Result<data::ContestFile> {
    Request::new(format!("{}/contest", url_prefix()))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}


pub async fn fetch_config() -> fetch::Result<data::configdata::ConfigContest> {
    Request::new(format!("{}/config", url_prefix()))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}
