use data::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_contest_file),
    components(schemas(ContestFile, Team, Problem, Answer))
)]
/// Animeitor api description.
pub struct ApiDoc;

impl ApiDoc {
    pub fn to_pretty_json() -> Result<String, serde_json::Error> {
        ApiDoc::openapi().to_pretty_json()
    }
}

#[utoipa::path(
        get,
        path = "/contest",
        responses(
            (status = 200, description = "Contest description", body = ContestFile),
        ),
        tag = "animeitor_api"
    )]
/// Gets the contest description.
pub fn get_contest_file() {}
