use data::{
    configdata::{ConfigContest, SedeEntry},
    *,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_contest_file, get_config_file, get_allruns_ws),
    components(schemas(ContestFile, Team, Problem, Answer, ConfigContest, SedeEntry, RunTuple))
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
    )]
/// Gets the contest description.
pub fn get_contest_file() {}

#[utoipa::path(
        get,
        path = "/config",
        responses(
            (status = 200, description = "Contest site configuration", body = ConfigContest),
        ),
    )]
/// Gets the contest description.
pub fn get_config_file() {}

#[utoipa::path(
        get,
        path = "/allruns_ws",
        responses(
            (status = 200, description = "WEBSOCKET: Get constest runs in a websocket connection", body = Vec<RunTuple>),
        ),
    )]
/// Returns all runs as a websocket connection.
/// API description is not exact, because response is a websocket connection.
pub fn get_allruns_ws() {}
