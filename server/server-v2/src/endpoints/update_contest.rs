use actix_web::{HttpRequest, HttpResponse, Responder, put, web};
use data::contest_state::ContestState;
use service::dbupdate_v2::update_runs_from_data;

use crate::app_data::AppData;

const API_KEY: &str = "apikey";

#[put("/contests")]
pub async fn update_contest(
    data: web::Data<AppData>,
    create_runs: web::Json<ContestState>,
    req: HttpRequest,
) -> impl Responder {
    let contest_key = match &data.server_api_key {
        Some(key) => key,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if req
        .headers()
        .get(API_KEY)
        .is_none_or(|k| k.as_bytes() != contest_key.as_bytes())
    {
        return HttpResponse::Unauthorized().finish();
    };

    let contest_state = create_runs.into_inner();

    match update_runs_from_data(contest_state, &data.shared_db, &data.runs_tx, &data.time_tx).await
    {
        Ok(()) => HttpResponse::Created().finish(),
        Err(e) => {
            tracing::error!(?e, "failed updating runs from data");

            HttpResponse::InternalServerError().finish()
        }
    }
}
