use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error with the jwt token")]
    JWTError(#[from] jsonwebtoken::errors::Error),
}