use actix_web::{HttpRequest, HttpResponse, Responder, post, put, web};
use sdk::{Contest, ContestParameters, ContestState, SiteConfiguration};

use crate::model::app::AppV2;

const API_KEY: &str = "apikey";

pub fn as_service(service_config: &mut web::ServiceConfig) {
    service_config.service((
        update_contest_parameters,
        update_contest_sites,
        update_contest_state,
        create_contest,
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

#[put("/contests/{contest}/state")]
pub async fn update_contest_state(
    data: web::Data<AppV2>,
    create_runs: web::Json<ContestState>,
    contest: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder, actix_web::Error> {
    authorize(&data, &req)?;

    let contest = data.get_contest(&contest).await?;

    contest.update_state(create_runs.into_inner()).await;

    Ok(HttpResponse::Created().finish())
}

#[put("/contests/{contest}/parameters")]
pub async fn update_contest_parameters(
    data: web::Data<AppV2>,
    parameters: web::Json<ContestParameters>,
    contest: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder, actix_web::Error> {
    authorize(&data, &req)?;

    let contest = data.get_contest(&contest).await?;

    contest.update_parameters(parameters.into_inner()).await;

    Ok(HttpResponse::Created().finish())
}

#[put("/contests/{contest}/sites")]
pub async fn update_contest_sites(
    data: web::Data<AppV2>,
    config: web::Json<SiteConfiguration>,
    contest: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder, actix_web::Error> {
    authorize(&data, &req)?;

    let contest = data.get_contest(&contest).await?;

    contest.update_site_configuration(config.into_inner()).await;

    Ok(HttpResponse::Created().finish())
}

#[post("/contests")]
pub async fn create_contest(
    data: web::Data<AppV2>,
    contest: web::Json<Contest>,
    req: HttpRequest,
) -> Result<impl Responder, actix_web::Error> {
    authorize(&data, &req)?;

    data.create_contest(contest.into_inner()).await?;

    Ok(HttpResponse::Created().finish())
}
