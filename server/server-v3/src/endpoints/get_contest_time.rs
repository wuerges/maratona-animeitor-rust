use actix_web::{HttpRequest, HttpResponse, get, web};
use tokio::pin;
use tokio_stream::StreamExt;
use tracing::{debug, instrument, warn};

use crate::{components::rejection::NotFound, model::app::AppV2};

#[utoipa::path(
    responses(
        (status = NOT_FOUND, description = "Contest was not found")
    ),
)]
#[instrument(skip_all)]
#[get("/contests/{contest}/time")]
pub async fn get_contest_time(
    data: web::Data<AppV2>,
    contest: web::Path<String>,
    body: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    debug!(?contest);
    let contest = data.get_contest(&contest).await.ok_or(NotFound)?;

    let (response, mut session, _msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        let runs = contest.get_time().await;
        pin!(runs);

        while let Some(time) = runs.next().await {
            match serde_json::to_string(&time) {
                Err(err) => warn!(?err, ?time, "failed serializing time"),
                Ok(run) => {
                    if let Err(err) = session.text(run).await {
                        debug!(?err, "websocket closed");
                        return;
                    }
                }
            }
        }
    });

    Ok(response)
}
