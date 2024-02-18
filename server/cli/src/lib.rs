use color_eyre::Section;
use data::configdata::{ConfigContest, ConfigSecret, Contest, Secret};
use serde::Deserialize;

pub mod test_revelation;

#[tracing::instrument(err)]
fn parse_config<T>(path: &std::path::Path) -> color_eyre::eyre::Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let text = std::fs::read_to_string(path)?;

    let config: T = toml::from_str(&text)?;

    Ok(config)
}

#[derive(clap::Args, Debug)]

pub struct SimpleArgs {
    #[clap(short = 's', long, default_value = "config/Default.toml")]
    /// Sets a custom config file
    pub sedes: String,

    #[clap(short = 'x', long)]
    /// Sets a custom config file
    pub secret: Option<String>,
}

impl SimpleArgs {
    #[tracing::instrument(err)]
    pub fn into_contest_and_secret(
        self,
    ) -> color_eyre::eyre::Result<(ConfigContest, Contest, Secret)> {
        let Self { sedes, secret } = self;

        let config = parse_config::<ConfigContest>(std::path::Path::new(&sedes))
            .map_err(|e| e.with_note(|| "Should be able to parse the config."))?;

        let contest = config.clone().into_contest();

        let config_secret = match secret {
            Some(secret) => parse_config::<ConfigSecret>(std::path::Path::new(&secret))
                .map_err(|e| e.with_note(|| "Should be able to parse secret file."))?,
            None => ConfigSecret::default(),
        }
        .into_secret(&contest);

        Ok((config, contest, config_secret))
    }
}
