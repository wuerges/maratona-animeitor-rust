use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error with the jwt token")]
    JWTError(#[from] jsonwebtoken::errors::Error),

    #[error("Wrog contest number: (exected: {0}, found: {1})")]
    WrongContestNumber(i32, i32),

    #[error("Wrog site number: (exected: {0}, found: {1})")]
    WrongSiteNumber(i32, i32),
}