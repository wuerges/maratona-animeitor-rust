use std::sync::Arc;

use data::configdata::ConfigSecrets;
use serde::Deserialize;
use service::DB;
use tokio::sync::Mutex;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection};

use crate::errors::Error;
use crate::routes::with_db;

pub fn serve_all_runs_secret(
    runs: Arc<Mutex<DB>>,
    secrets: Box<ConfigSecrets>,
) -> BoxedFilter<(String,)> {
    with_db(runs)
        .and(warp::any().map(move || secrets.clone()))
        .and(warp::query::<SecretQuery>())
        .and_then(serve_all_runs_secret_filter)
        .boxed()
}

#[derive(Deserialize)]
struct SecretQuery {
    secret: Option<String>,
}

async fn serve_all_runs_secret_filter(
    runs: Arc<Mutex<DB>>,
    secrets: Box<ConfigSecrets>,
    query: SecretQuery,
) -> Result<String, Rejection> {
    Ok(serve_all_runs_secret_service(runs, secrets, query).await?)
}

async fn serve_all_runs_secret_service(
    runs: Arc<Mutex<DB>>,
    secrets: Box<ConfigSecrets>,
    query: SecretQuery,
) -> Result<String, Error> {
    match query.secret.and_then(|secret| secrets.secrets.get(&secret)) {
        Some(team_patterns) => {
            let db = runs.lock().await;
            Ok(serde_json::to_string(
                &db.run_file_secret.filter_team_patterns(team_patterns),
            )?)
        }
        None => Err(Error::InvalidSecret),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_serve_timer_ws() {
//         let (time_tx, _): (broadcast::Sender<TimerData>, _) = broadcast::channel(1000000);
//         let send_time_tx = time_tx.clone();

//         let timer = warp::path("timer").and(serve_timer(time_tx));

//         let expected1 = Message::text(serde_json::to_string(&TimerData::new(1, 2)).unwrap());
//         let expected2 = Message::text(serde_json::to_string(&TimerData::new(2, 2)).unwrap());

//         let mut client = warp::test::ws()
//             .path("/timer")
//             .handshake(timer.clone())
//             .await
//             .expect("handshake");

//         {
//             let mut client2 = warp::test::ws()
//                 .path("/timer")
//                 .handshake(timer)
//                 .await
//                 .expect("handshake");

//             send_time_tx
//                 .send(TimerData::new(1, 2))
//                 .expect("to send message");

//             assert_eq!(client2.recv().await.expect("to receive message"), expected1);
//         }

//         assert_eq!(client.recv().await.expect("to receive message"), expected1);

//         send_time_tx
//             .send(TimerData::new(2, 2))
//             .expect("to send message");

//         assert_eq!(client.recv().await.expect("to receive message"), expected2);
//     }
// }

// let secrets = Box::new(ConfigSecrets {
//     secrets: Box::new(
//         [("secretsecret".to_string(), vec!["teambrbr1".to_string()])]
//             .into_iter()
//             .collect(),
//     ),
// });
