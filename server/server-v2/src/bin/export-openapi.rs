//! Export exactly the specifications served by the API, without starting a server.
use std::{fs, path::PathBuf};
use utoipa::OpenApi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("usage: export-openapi OUTPUT_DIRECTORY")?,
    );
    fs::create_dir_all(&output)?;
    for (name, spec) in [
        (
            "internal.json",
            server_v2::openapi::InternalApiDoc::openapi(),
        ),
        ("public.json", server_v2::openapi::PublicApiDoc::openapi()),
    ] {
        // Sorting via serde_json::Value makes snapshots independent of map insertion order.
        let value = serde_json::to_value(spec)?;
        fs::write(
            output.join(name),
            format!("{}\n", serde_json::to_string_pretty(&value)?),
        )?;
    }
    Ok(())
}
