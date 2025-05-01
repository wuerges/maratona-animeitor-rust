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
    Ok(std::fs::read(path)?)
}

async fn read_bytes_from_url(uri: &str) -> Result<Vec<u8>, reqwest::Error> {
    let resp = reqwest::get(uri).await?.bytes().await?;

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
    try_read_from_zip(zip, name)
        .or_else(|_| try_read_from_zip(zip, &format!("./{}", name)))
        .or_else(|_| try_read_from_zip(zip, &format!("./sample/{}", name)))
        .or_else(|_| try_read_from_zip(zip, &format!("sample/{}", name)))
        .or_else(|_| try_read_from_zip(zip, &format!("./webcast/{}", name)))
        .or_else(|_| try_read_from_zip(zip, &format!("webcast/{}", name)))
}

pub async fn load_data_from_url_maybe(
    uri: &str,
) -> ServiceResult<(i64, data::ContestFile, data::RunsFile)> {
    let zip_data = read_bytes_from_path(&uri).await?;

    let reader = std::io::Cursor::new(&zip_data);
    let mut zip = zip::ZipArchive::new(reader)?;

    let time_data: i64 = read_from_zip(&mut zip, "time")?.parse()?;

    let contest_data = read_from_zip(&mut zip, "contest")?;
    let contest_data = read_contest(&contest_data)?;

    let runs_data = read_from_zip(&mut zip, "runs")?;
    let runs_data = read_runs(&runs_data)?;

    Ok((time_data, contest_data, runs_data))
}
