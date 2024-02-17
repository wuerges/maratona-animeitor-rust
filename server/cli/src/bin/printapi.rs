use server::openapi::ApiDoc;

fn main() -> color_eyre::eyre::Result<()> {
    println!("{}", ApiDoc::to_pretty_json()?);

    Ok(())
}
