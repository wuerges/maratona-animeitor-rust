use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../client/www"]
pub struct ClientAssets;
