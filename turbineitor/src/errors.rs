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

    #[error("Expected Some(token), got None.")]
    EmptyToken,

    #[error(transparent)]
    JsonEncode(#[from] serde_json::Error),

    #[error(transparent)]
    DieselError(#[from] diesel::result::Error),

    #[error(transparent)]
    ContestError(#[from] maratona_animeitor_rust::data::ContestError),
}

use warp::reject::Reject;

impl Reject for Error {}
