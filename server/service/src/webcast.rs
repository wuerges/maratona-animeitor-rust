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

    let resp = CLIENT
        .get(uri)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    Ok(resp.into())
}

/// A diagnostic safe for logs: never includes source URLs, credentials, or payloads.
pub fn failure_summary(error: &crate::errors::Error) -> String {
    use crate::errors::Error;
    match error {
        Error::Fetch(error) => {
            if error.path.starts_with("https://") || error.path.starts_with("http://") {
                let request = &error.reqwest_err;
                if let Some(status) = request.status() {
                    format!("webcast server returned HTTP {status}; check the source URL and access permissions")
                } else if request.is_timeout() {
                    "webcast request timed out; check connectivity from the feeder container".into()
                } else if request.is_connect() {
                    "could not connect to webcast server; check DNS, network access, and TLS certificates from the feeder container".into()
                } else {
                    "webcast HTTP request failed; check the source URL, redirects, and response transfer".into()
                }
            } else {
                format!("could not read local webcast ZIP ({:?}); check that the file is mounted in the feeder container and readable", error.file_err.kind())
            }
        }
        Error::ZipError(_) => "source is not a readable ZIP archive; check that the URL returns a webcast ZIP rather than an HTML login or error page".into(),
        Error::WebcastZipError(ZipErr::ZipError { file, .. }) => {
            format!("could not read required webcast ZIP entry {file}; expected time, contest, and runs files")
        }
        Error::WebcastZipError(ZipErr::Utf8(_)) => "webcast ZIP contains text that is not valid UTF-8".into(),
        Error::WebcastZipError(ZipErr::Io(_)) | Error::IO(_) => "failed reading webcast ZIP contents; check archive integrity".into(),
        Error::ParseInt(_) => "invalid integer in webcast time, contest, or runs data".into(),
        Error::ContestFileParse(field) => format!("webcast contest file is missing {field}"),
        Error::InvalidAnswer(_) => "webcast runs contain an unsupported answer code".into(),
        Error::BadLetter(_) => "webcast runs contain an invalid problem identifier".into(),
        Error::Parse(_) => "webcast contest or runs data does not match the expected BOCA format".into(),
        Error::Database(_) => "database failure while loading webcast".into(),
    }
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
    async fn http_errors_report_status_without_private_url() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            socket.read(&mut request).await.unwrap();
            socket
                .write_all(
                    b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
        });
        let error = load_data_from_url_maybe(&format!(
            "http://{address}/private-secret?token=secret-token"
        ))
        .await
        .unwrap_err();
        let summary = failure_summary(&error);
        assert!(summary.contains("HTTP 403"), "{summary}");
        assert!(!summary.contains("private-secret"));
        assert!(!summary.contains("secret-token"));
        assert!(!summary.contains('\n'));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn missing_local_zip_explains_container_mount() {
        let path =
            std::env::temp_dir().join(format!("missing-webcast-{}-secret.zip", std::process::id()));
        let error = load_data_from_url_maybe(path.to_str().unwrap())
            .await
            .unwrap_err();
        let summary = failure_summary(&error);
        assert!(summary.contains("NotFound"));
        assert!(summary.contains("mounted in the feeder container"));
        assert!(!summary.contains("secret.zip"));
    }

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
