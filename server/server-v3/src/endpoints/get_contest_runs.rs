use actix_web::{HttpRequest, HttpResponse, error::ErrorUnauthorized, get, web};
use tokio::pin;
use tokio_stream::StreamExt;
use tracing::{debug, instrument, warn};

use crate::{
    components::rejection::NotFoundContest,
    model::app::{AppV2, ContestApp},
};

const SECRET: &str = "x-secret";

async fn check_secret(contest: &ContestApp, req: &HttpRequest) -> Result<(), actix_web::Error> {
    let secret = contest
        .get_secret()
        .await
        .ok_or_else(|| ErrorUnauthorized("contest secret not set"))?;

    let value = req
        .headers()
        .get(SECRET)
        .ok_or_else(|| ErrorUnauthorized("missing secret in request"))?;

    if secret.as_bytes() == value.as_bytes() {
        Ok(())
    } else {
        Err(ErrorUnauthorized("incorrect secret"))
    }
}

/// Create a websocket connection to return all runs of the current contest.
#[utoipa::path(responses(NotFoundContest))]
#[instrument(skip_all)]
#[get("/contests/{contest}/runs-unmasked-websocket")]
pub async fn get_contest_runs_unmasked(
    data: web::Data<AppV2>,
    contest: web::Path<String>,
    body: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    debug!(?contest);
    let contest = data.get_contest(&contest).await.ok_or(NotFoundContest)?;

    check_secret(&contest, &req).await?;

    let (response, mut session, _msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        let runs = contest.get_runs_unmasked().await;
        pin!(runs);

        while let Some(batch) = runs.next().await {
            let batch = batch.await;

            for run in batch {
                match serde_json::to_string(&run) {
                    Err(err) => warn!(?err, ?run, "failed serializing run"),
                    Ok(run) => {
                        if let Err(err) = session.text(run).await {
                            debug!(?err, "websocket closed");
                            return;
                        }
                    }
                }
            }
        }
    });

    Ok(response)
}

/// Create a websocket connection to return all runs of the current contest, masked if the contest is frozen.
#[utoipa::path(responses(NotFoundContest))]
#[instrument(skip_all)]
#[get("/contests/{contest}/runs-websocket")]
pub async fn get_contest_runs(
    data: web::Data<AppV2>,
    contest: web::Path<String>,
    body: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    debug!(?contest);
    let contest = data.get_contest(&contest).await.ok_or(NotFoundContest)?;

    let (response, mut session, _msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        let runs = contest.get_runs_masked().await;
        pin!(runs);

        while let Some(batch) = runs.next().await {
            let batch = batch.await;

            for run in batch {
                match serde_json::to_string(&run) {
                    Err(err) => warn!(?err, ?run, "failed serializing run"),
                    Ok(run) => {
                        if let Err(err) = session.text(run).await {
                            debug!(?err, "websocket closed");
                            return;
                        }
                    }
                }
            }
        }
    });

    Ok(response)
}
