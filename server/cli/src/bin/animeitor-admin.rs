use clap::Parser;
use cli::admin::{self, args::AdminArgs};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let args = AdminArgs::parse();
    let json_mode = args.json;
    match admin::run(args).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            if json_mode {
                let envelope = error.downcast_ref::<admin::AdminError>()
                    .map(|e| e.envelope.clone())
                    .unwrap_or_else(|| serde_json::json!({"errors":[{"code":"client_error","message":error.to_string()}]}));
                eprintln!("{envelope}");
            } else {
                eprintln!("{error}");
            }
            std::process::ExitCode::FAILURE
        }
    }
}
