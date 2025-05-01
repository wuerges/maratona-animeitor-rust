use thiserror::Error;

use crate::webcast::{FetchErr, ZipErr};

pub type ServiceResult<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Fetch(#[from] FetchErr),

    #[error(transparent)]
    WebcastZipError(#[from] ZipErr),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),

    #[error("Error sending data after DB refresh: {0}")]
    SendError(String),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Invalid Answer: {0}")]
    InvalidAnswer(String),

    #[error("Could not parse contest file: {0}")]
    ContestFileParse(&'static str),

    #[error(transparent)]
    Chain(#[from] data::ContestError),

    #[error(transparent)]
    ConfigParse(#[from] toml::de::Error),

    #[error("Error::Parse: {0}")]
    Parse(String),
}
