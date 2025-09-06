use actix_web::{HttpRequest, HttpResponse, Responder, put, web};
use data::{ContestFile, RunTuple, RunsFile};
use serde::Deserialize;
use service::dbupdate_v2::update_runs_from_data;

use crate::app_data::AppData;

#[derive(Deserialize, Debug)]
struct ContestState {
    runs: Vec<RunTuple>,
    time: data::TimeFile,
    contest: ContestFile,
}

const API_KEY: &str = "apikey";

fn authorize(data: &web::Data<AppData>, req: &HttpRequest) -> Result<(), actix_web::Error> {
    let contest_key = match &data.server_api_key {
        Some(key) => key,
        None => return Err(actix_web::error::ErrorUnauthorized("missing credentials")),
    };

    if req
        .headers()
        .get(API_KEY)
        .is_none_or(|k| k.as_bytes() != contest_key.as_bytes())
    {
        return Err(actix_web::error::ErrorUnauthorized("incorrect credentials"));
    };

    Ok(())
}

#[put("/contests")]
pub async fn update_contest(
    data: web::Data<AppData>,
    create_runs: web::Json<ContestState>,
    req: HttpRequest,
) -> Result<impl Responder, actix_web::Error> {
    authorize(&data, &req)?;

    let ContestState {
        runs,
        time,
        contest,
    } = create_runs.into_inner();

    let run_file = RunsFile::new(runs);

    match update_runs_from_data(
        (time, contest, run_file),
        &data.shared_db,
        &data.runs_tx,
        &data.time_tx,
    )
    .await
    {
        Ok(()) => Ok(HttpResponse::Created().finish()),
        Err(e) => {
            tracing::error!(?e, "failed updating runs from data");

            Err(actix_web::error::ErrorInternalServerError(
                "failed updating runs",
            ))
        }
    }
}
