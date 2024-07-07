use std::collections::HashMap;

use color_eyre::Section;
use data::configdata::{ConfigContest, ConfigSecret, Contest, Secret};
use serde::Deserialize;
use service::pair_arg::{FromPairArg, PairArg};

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

#[derive(Debug, Clone)]
pub struct NamedSede {
    file: String,
    name: String,
}

impl From<PairArg> for NamedSede {
    fn from(PairArg { first, second }: PairArg) -> Self {
        Self {
            name: second,
            file: first,
        }
    }
}

#[derive(clap::Args, Debug)]

pub struct SimpleArgs {
    #[clap(short = 's', long, default_value = "config/basic.toml:default")]
    /// Sets a custom config file
    pub sedes: Vec<FromPairArg<NamedSede>>,

    #[clap(short = 'y', long)]
    /// Salt to be added to secrets
    pub salt: Option<String>,

    #[clap(short = 'x', long)]
    /// Sets custom secrets file
    pub secret: Vec<String>,
}

fn gather_secrets(secrets: &[String]) -> color_eyre::Result<ConfigSecret> {
    let mut empty = ConfigSecret { secrets: vec![] };
    for path in secrets {
        let secret = parse_config::<ConfigSecret>(std::path::Path::new(path))
            .map_err(|e| e.with_note(|| "Should be able to parse secret file."))?;

        empty.secrets.extend(secret.secrets);
    }
    Ok(empty)
}

impl SimpleArgs {
    #[tracing::instrument(err)]
    pub fn into_contest_and_secret(
        self,
    ) -> color_eyre::eyre::Result<HashMap<String, (ConfigContest, Contest, Secret)>> {
        let Self {
            salt,
            sedes,
            secret,
        } = self;

        let main_config_secret = gather_secrets(&secret)?;

        let mut result = HashMap::new();

        for sede in sedes {
            let NamedSede { file, name } = sede.into_inner();
            let config = parse_config::<ConfigContest>(std::path::Path::new(&file))
                .map_err(|e| e.with_note(|| "Should be able to parse the config."))?;

            let contest = config.clone().into_contest();
            let secret = main_config_secret.into_secret(salt.clone(), &contest);

            result.insert(name, (config, contest, secret));
        }

        Ok(result)
    }
}
