use thiserror::Error;

pub type ServiceResult<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct SerializationError(pub serde_json::Error);

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    InvalidUri(#[from] hyper::http::uri::InvalidUri),

    #[error(transparent)]
    Hyper(#[from] hyper::Error),

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

    #[error("Error: {0}")]
    Info(String),

    #[error("Error::Parse: {0}")]
    Parse(String),
}
