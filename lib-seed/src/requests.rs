use seed::prelude::*;
use maratona_animeitor_rust::data;


pub async fn fetch_allruns(source :&String) -> fetch::Result<data::RunsFile> {
    Request::new(format!("/{}/allruns", source))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub async fn fetch_allruns_secret(source :&String, secret : &String) -> fetch::Result<data::RunsFile> {
    // Request::new("/allruns")
    Request::new(format!("/{}/allruns_{}", source, secret))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}


pub async fn fetch_contest(source :&String) -> fetch::Result<data::ContestFile> {
    Request::new(format!("/{}/contest", source))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub async fn fetch_runspanel(source :&String) -> fetch::Result<Vec<data::RunsPanelItem>> {
    Request::new(format!("/{}/runs", source))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}


pub async fn fetch_time_file(source :&String) -> fetch::Result<data::TimeFile> {
    Request::new(format!("/{}/timer", source))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}
