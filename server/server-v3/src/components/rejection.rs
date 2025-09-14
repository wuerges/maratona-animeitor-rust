use utoipa::IntoResponses;

#[derive(thiserror::Error, Debug, IntoResponses)]
#[response(status = 404, description = "contest not found")]
#[error("contest not found")]
pub struct NotFoundContest;

#[derive(thiserror::Error, Debug, IntoResponses)]
#[response(status = 404, description = "site configuration not found")]
#[error("site configuration not found")]
pub struct NotFoundSite;

#[derive(thiserror::Error, Debug, IntoResponses)]
#[response(status = 409, description = "contest already exists")]
#[error("conflict: contest already exists")]
pub struct ConflictContest;

impl From<NotFoundContest> for actix_web::Error {
    fn from(value: NotFoundContest) -> Self {
        actix_web::error::ErrorNotFound(value)
    }
}

impl From<NotFoundSite> for actix_web::Error {
    fn from(value: NotFoundSite) -> Self {
        actix_web::error::ErrorNotFound(value)
    }
}

impl From<ConflictContest> for actix_web::Error {
    fn from(value: ConflictContest) -> Self {
        actix_web::error::ErrorConflict(value)
    }
}

#[derive(thiserror::Error, Debug, IntoResponses)]
#[response(status = 401)]
#[error("unauthorized: {reason}")]
pub struct Unauthorized {
    pub reason: String,
}

impl From<Unauthorized> for actix_web::Error {
    fn from(value: Unauthorized) -> Self {
        actix_web::error::ErrorUnauthorized(value)
    }
}
