use crate::dataio::{read_contest, read_runs};
use crate::errors::{CResult, Error};
use hyper::{body, Client};
use hyper_tls::HttpsConnector;
use std::io::Read;
use zip;

async fn read_bytes_from_path(path: &str) -> CResult<Vec<u8>> {
    read_bytes_from_url(path)
        .await
        .or_else(|_| read_bytes_from_file(path))
}

fn read_bytes_from_file(path: &str) -> CResult<Vec<u8>> {
    Ok(std::fs::read(path)?)
}

async fn read_bytes_from_url(uri: &str) -> CResult<Vec<u8>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let uri = uri.parse()?;

    let resp = client.get(uri).await?;
    let bytes = body::to_bytes(resp.into_body()).await?;
    Ok(bytes.to_vec())
}

fn try_read_from_zip(
    zip: &mut zip::ZipArchive<std::io::Cursor<&std::vec::Vec<u8>>>,
    name: &str,
) -> CResult<String> {
    let mut runs_zip = zip
        .by_name(name)
        .map_err(|e| Error::Info(format!("Could not unpack file: {} {:?}", name, e)))?;
    let mut buffer = Vec::new();
    runs_zip.read_to_end(&mut buffer)?;
    let runs_data = String::from_utf8(buffer)
        .map_err(|_| Error::Info("Could not parse to UTF8".to_string()))?;
    Ok(runs_data)
}

fn read_from_zip(
    zip: &mut zip::ZipArchive<std::io::Cursor<&std::vec::Vec<u8>>>,
    name: &str,
) -> CResult<String> {
    try_read_from_zip(zip, name)
        .or_else(|_| try_read_from_zip(zip, &format!("./{}", name)))
        .or_else(|_| try_read_from_zip(zip, &format!("./sample/{}", name)))
        .or_else(|_| try_read_from_zip(zip, &format!("sample/{}", name)))
        .or_else(|_| try_read_from_zip(zip, &format!("./webcast/{}", name)))
        .or_else(|_| try_read_from_zip(zip, &format!("webcast/{}", name)))
}

pub async fn load_data_from_url_maybe(
    uri: String,
) -> CResult<(i64, data::ContestFile, data::RunsFile)> {
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
