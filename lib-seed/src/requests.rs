use seed::prelude::*;
use maratona_animeitor_rust::data;

fn prepend(url : &str, source:&Option<String>) -> String {
    source.clone().map( |s| format!("/{}{}", s, url) ).unwrap_or(url.to_string())
}


pub async fn fetch_allruns(source :&Option<String>) -> fetch::Result<data::RunsFile> {
    Request::new(prepend("/allruns", source))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub async fn fetch_allruns_secret(source :&Option<String>, secret : &String) -> fetch::Result<data::RunsFile> {
    // Request::new("/allruns")
    Request::new(prepend(format!("/allruns_{}", secret).as_str(), source))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}


pub async fn fetch_contest(source :&Option<String>) -> fetch::Result<data::ContestFile> {
    Request::new(prepend("/contest", source))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub async fn fetch_runspanel(source :&Option<String>) -> fetch::Result<Vec<data::RunsPanelItem>> {
    Request::new(prepend("/runs", source))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}


pub async fn fetch_time_file(source :&Option<String>) -> fetch::Result<data::TimerData> {
    Request::new(prepend("/timer", source))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}
