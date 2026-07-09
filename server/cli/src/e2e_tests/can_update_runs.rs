use serde_json::json;
use tracing::{debug, error};

#[tokio::test]
async fn test_update_contest_state() -> color_eyre::Result<()> {
    tracing_subscriber::fmt().try_init().ok();
    let client = reqwest::Client::new();
    let server_url = "http://localhost:8000/api";
    let server_api_key = "test-key";

    let input = json!({
        "runs": [],
        "time": 34,
        "contest": {
            "contest_name": "Test Contest",
            "teams": {
                "teambrbr1": {
                    "login": "teambrbr1",
                    "escola": "escola 1",
                    "name": "name 1",
                    "placement": 1,
                    "placement_global": 1,
                    "problems": {
                        "A": {
                            "solved": false,
                            "solved_first": false,
                            "submissions": 3,
                            "penalty": 50,
                            "time_solved": 123,
                            "answers": [
                                { "Yes": { "time": 1, "is_first": false, "run_id": 1 }}
                            ],
                            "waits": [1],
                            "id": 19,
                        }
                    },
                    "id": 191,
                }
            },
            "current_time": 34,
            "maximum_time": 1234,
            "score_freeze_time": 123,
            "penalty_per_wrong_answer": 20,
            "number_problems": 5
        }
    });

    let result = client
        .put(format!("{server_url}/contests"))
        .json(&input)
        .header("apikey", server_api_key)
        .send()
        .await?;

    match result.error_for_status() {
        Ok(_) => {
            debug!("ok");
        }
        Err(err) => error!(?err, "status error"),
    }

    Ok(())
}
