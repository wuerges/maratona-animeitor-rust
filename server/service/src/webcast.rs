use crate::contest_state::ContestState;
use crate::dataio::{read_contest, read_runs};
use crate::errors::ServiceResult;
use std::io::Read;
use std::string::FromUtf8Error;
use thiserror::Error;
use zip;

#[derive(Debug, Error)]
#[error(
    "failed to read from BOCA_URL: {:?}\n{}\n{}",
    path,
    reqwest_err,
    file_err
)]
pub struct FetchErr {
    path: String,
    file_err: std::io::Error,
    reqwest_err: reqwest::Error,
}

async fn read_bytes_from_path(path: &str) -> Result<Vec<u8>, FetchErr> {
    read_bytes_from_url(path).await.or_else(|reqwest_err| {
        read_bytes_from_file(path).map_err(|file_err| FetchErr {
            path: path.to_string(),
            file_err,
            reqwest_err,
        })
    })
}

fn read_bytes_from_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(path)
}

async fn read_bytes_from_url(uri: &str) -> Result<Vec<u8>, reqwest::Error> {
    static CLIENT: std::sync::LazyLock<reqwest::Client> =
        std::sync::LazyLock::new(reqwest::Client::new);

    let resp = CLIENT.get(uri).send().await?.bytes().await?;

    Ok(resp.into())
}

#[derive(Debug, Error)]
pub enum ZipErr {
    #[error("failed to unpack file: {}\n{}", file, error)]
    ZipError {
        file: String,
        error: zip::result::ZipError,
    },
    #[error("failed to read buffer:\n{}", .0)]
    Io(#[from] std::io::Error),
    #[error("failed to parse utf8:\n{}", .0)]
    Utf8(#[from] FromUtf8Error),
}

fn try_read_from_zip(
    zip: &mut zip::ZipArchive<std::io::Cursor<&std::vec::Vec<u8>>>,
    name: &str,
) -> Result<String, ZipErr> {
    let mut runs_zip = zip.by_name(name).map_err(|error| ZipErr::ZipError {
        file: name.to_string(),
        error,
    })?;
    let mut buffer = Vec::new();
    runs_zip.read_to_end(&mut buffer)?;
    let runs_data = String::from_utf8(buffer)?;
    Ok(runs_data)
}

fn read_from_zip(
    zip: &mut zip::ZipArchive<std::io::Cursor<&std::vec::Vec<u8>>>,
    name: &str,
) -> Result<String, ZipErr> {
    // BOCA zips may store entries at the root or under sample/ or webcast/,
    // with or without a ./ prefix.
    let mut last_err = None;
    for prefix in ["", "./", "./sample/", "sample/", "./webcast/", "webcast/"] {
        match try_read_from_zip(zip, &format!("{prefix}{name}")) {
            Ok(data) => return Ok(data),
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.unwrap())
}

pub async fn load_data_from_url_maybe(uri: &str) -> ServiceResult<ContestState> {
    let zip_data = read_bytes_from_path(uri).await?;

    let reader = std::io::Cursor::new(&zip_data);
    let mut zip = zip::ZipArchive::new(reader)?;

    // The `time` file is already in seconds; contest timings and run times
    // come in minutes. Internally every time is seconds (doc/public-api.md).
    let time_data: i64 = read_from_zip(&mut zip, "time")?.parse()?;

    let contest_data = read_from_zip(&mut zip, "contest")?;
    let mut contest_data = read_contest(&contest_data)?;
    contest_data.maximum_time *= 60;
    contest_data.current_time *= 60;
    contest_data.score_freeze_time *= 60;
    contest_data.penalty_per_wrong_answer *= 60;

    let runs_data = read_from_zip(&mut zip, "runs")?;
    let mut runs_data = read_runs(&runs_data)?;
    for run in &mut runs_data {
        run.time *= 60;
        if let data::Answer::Yes { time, .. } = &mut run.answer {
            *time *= 60;
        }
    }

    Ok(ContestState {
        runs: runs_data,
        time: time_data,
        contest: contest_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn moj_webcast_times_become_seconds() -> ServiceResult<()> {
        let state = load_data_from_url_maybe(
            "../../tests/inputs/1_fase_2026/webcast-final-depois-do-contest-moj.zip",
        )
        .await?;

        // The `time` file is seconds already: 5h contest.
        assert_eq!(state.time, 18000);
        // Contest timings come in minutes: 300min -> 18000s, penalty 20min -> 1200s.
        assert_eq!(state.contest.maximum_time, 18000);
        assert_eq!(state.contest.current_time, 18000);
        assert_eq!(state.contest.score_freeze_time, 18000);
        assert_eq!(state.contest.penalty_per_wrong_answer, 1200);
        // Runs come in minutes: the first run (1min) becomes 60s, including
        // the time inside the Yes answer.
        let first = state.runs.first().expect("the MOJ zip has runs");
        assert_eq!(first.time, 60);
        match &first.answer {
            data::Answer::Yes { time, .. } => assert_eq!(*time, 60),
            other => panic!("expected a Yes answer, got {other:?}"),
        }
        Ok(())
    }
}
