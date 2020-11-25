use thiserror::Error;


pub type CResult<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    InvalidUri(#[from] warp::http::uri::InvalidUri),

    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    
    #[error("Invalid Answer: {0}")]
    InvalidAnswer(String),
    
    #[error(transparent)]
    Chain(#[from] data::ContestError),
    
    #[error("Error: {0}")]
    Info(String),
}