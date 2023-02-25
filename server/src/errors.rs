use thiserror::Error;
use warp::reject::Reject;

pub type CResult<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),
    #[error(transparent)]
    ServiceError(#[from] service::errors::Error),
    #[error("invalid secret")]
    InvalidSecret,
}

impl Reject for Error {}
