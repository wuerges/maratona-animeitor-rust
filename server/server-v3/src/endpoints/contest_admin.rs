use actix_web::{HttpRequest, post, put, web};
use sdk::{Contest, ContestParameters, ContestState, SiteConfiguration};

use crate::{
    components::{
        rejection::{NotFoundContest, Unauthorized},
        success::Success,
    },
    endpoints::api::open_api_internal,
    model::app::AppV2,
};

const API_KEY: &str = "x-api-key";

pub fn as_service(service_config: &mut web::ServiceConfig) {
    service_config.service((
        update_contest_parameters,
        update_contest_sites,
        update_contest_state,
        create_contest,
        open_api_internal,
    ));
}

fn authorize(data: &web::Data<AppV2>, req: &HttpRequest) -> Result<(), actix_web::Error> {
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

#[utoipa::path(
    responses(NotFoundContest, Success, Unauthorized),
    context_path = "/internal"
)]
#[put("/contests/{contest}/state")]
pub async fn update_contest_state(
    data: web::Data<AppV2>,
    create_runs: web::Json<ContestState>,
    contest: web::Path<String>,
    req: HttpRequest,
) -> Result<Success, actix_web::Error> {
    authorize(&data, &req)?;

    let contest = data.get_contest(&contest).await.ok_or(NotFoundContest)?;

    contest.update_state(create_runs.into_inner()).await;

    Ok(Success)
}

#[utoipa::path(
    responses(NotFoundContest, Success, Unauthorized),
    context_path = "/internal"
)]
#[put("/contests/{contest}/secret")]
pub async fn update_contest_secret(
    data: web::Data<AppV2>,
    secret: web::Json<sdk::ContestSecret>,
    contest: web::Path<String>,
    req: HttpRequest,
) -> Result<Success, actix_web::Error> {
    authorize(&data, &req)?;

    let contest = data.get_contest(&contest).await.ok_or(NotFoundContest)?;

    contest.set_secret(&secret.secret).await;

    Ok(Success)
}

#[utoipa::path(
    responses(NotFoundContest, Success, Unauthorized),
    context_path = "/internal"
)]
#[put("/contests/{contest}/parameters")]
pub async fn update_contest_parameters(
    data: web::Data<AppV2>,
    parameters: web::Json<ContestParameters>,
    contest: web::Path<String>,
    req: HttpRequest,
) -> Result<Success, actix_web::Error> {
    authorize(&data, &req)?;

    let contest = data.get_contest(&contest).await.ok_or(NotFoundContest)?;

    contest.update_parameters(parameters.into_inner()).await;

    Ok(Success)
}

#[utoipa::path(
    responses(NotFoundContest, Success, Unauthorized),
    context_path = "/internal"
)]
#[put("/contests/{contest}/sites")]
pub async fn update_contest_sites(
    data: web::Data<AppV2>,
    config: web::Json<SiteConfiguration>,
    contest: web::Path<String>,
    req: HttpRequest,
) -> Result<Success, actix_web::Error> {
    authorize(&data, &req)?;

    let contest = data.get_contest(&contest).await.ok_or(NotFoundContest)?;

    contest.update_site_configuration(config.into_inner()).await;

    Ok(Success)
}

#[utoipa::path(
    responses(NotFoundContest, Success, Unauthorized),
    context_path = "/internal"
)]
#[post("/contests")]
pub async fn create_contest(
    data: web::Data<AppV2>,
    contest: web::Json<Contest>,
    req: HttpRequest,
) -> Result<Success, actix_web::Error> {
    authorize(&data, &req)?;

    data.create_contest(contest.into_inner()).await?;

    Ok(Success)
}
