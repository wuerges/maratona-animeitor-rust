use actix_web::{HttpRequest, HttpResponse, Responder, get, put, web};
use serde::Deserialize;

use crate::model::app::AppV2;

#[derive(Deserialize, Debug)]
pub struct ContestState {
    pub runs: Vec<sdk::Run>,
    pub time: sdk::Time,
}

#[derive(Deserialize, Debug)]
pub struct ContestConfig {
    pub config: ContestFile,
}

const API_KEY: &str = "apikey";

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

#[put("/contests/{contest}/config")]
pub async fn update_contest_config(
    data: web::Data<AppV2>,
    config: web::Json<ContestFile>,
    contest: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder, actix_web::Error> {
    authorize(&data, &req)?;

    let contest = data.get_contest(&contest).await?;

    contest.update_config(config.into_inner()).await;

    Ok(HttpResponse::Created().finish())
}

#[put("/contests/{contest}/sedes")]
pub async fn update_contest_sedes(
    data: web::Data<AppV2>,
    config: web::Json<ConfigContest>,
    contest: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder, actix_web::Error> {
    authorize(&data, &req)?;

    let contest = data.get_contest(&contest).await?;

    contest.update_sedes(config.into_inner()).await;

    Ok(HttpResponse::Created().finish())
}
