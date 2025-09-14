#[derive(thiserror::Error, Debug)]
#[error("not found")]
pub struct NotFound;

#[derive(thiserror::Error, Debug)]
#[error("not found")]
pub struct Conflict;

impl From<NotFound> for actix_web::Error {
    fn from(value: NotFound) -> Self {
        actix_web::error::ErrorNotFound(value)
    }
}

impl From<Conflict> for actix_web::Error {
    fn from(value: Conflict) -> Self {
        actix_web::error::ErrorConflict(value)
    }
}
