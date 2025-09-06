#[derive(thiserror::Error, Debug)]
pub enum Rejection {
    #[error(transparent)]
    NotFound(#[from] NotFound),
    #[error(transparent)]
    Conflict(#[from] Conflict),
}

#[derive(thiserror::Error, Debug)]
#[error("not found")]
pub struct NotFound;

#[derive(thiserror::Error, Debug)]
#[error("not found")]
pub struct Conflict;
