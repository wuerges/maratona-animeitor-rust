use seed::prelude::*;
use data;

pub async fn fetch_allruns() -> fetch::Result<data::RunsFile> {
    Request::new("/allruns")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub async fn fetch_allruns_secret(secret : &String) -> fetch::Result<data::RunsFile> {
    // Request::new("/allruns")
    Request::new(format!("/allruns_{}", secret))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}


pub async fn fetch_contest() -> fetch::Result<data::ContestFile> {
    Request::new("/contest")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub async fn fetch_runspanel() -> fetch::Result<Vec<data::RunsPanelItem>> {
    Request::new("/runs")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}


pub async fn fetch_time_file() -> fetch::Result<data::TimerData> {
    Request::new("/timer")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}
