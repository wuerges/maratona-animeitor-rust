use thiserror::Error;
use serde_json;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    JWTError(#[from] jsonwebtoken::errors::Error),

    #[error("Wrog contest number: (exected: {0}, found: {1})")]
    WrongContestNumber(i32, i32),

    #[error("Wrog site number: (exected: {0}, found: {1})")]
    WrongSiteNumber(i32, i32),

    #[error("User not found {0}")]
    UserNotFound(String),

    #[error("Wrong password.")]
    WrongPassword,

    #[error("Expected Some({0}), got None.")]
    Empty(String),

    #[error(transparent)]
    JsonEncode(#[from] serde_json::Error),

    #[error(transparent)]
    DieselError(#[from] diesel::result::Error),

    #[error(transparent)]
    ContestError(#[from] maratona_animeitor_rust::data::ContestError),

    #[error(transparent)]
    PoolError(#[from] r2d2::Error)
}

impl Error {
    pub fn empty(t: &str) -> Self {
        Error::Empty(t.to_string())
    }
}

use warp::reject::Reject;

impl Reject for Error {}
