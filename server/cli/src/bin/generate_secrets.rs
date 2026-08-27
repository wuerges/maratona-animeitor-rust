use std::{collections::HashSet, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::WrapErr;
use data::configdata::ConfigContest;
use rand::distr::{Alphanumeric, SampleString};
use service::config_secret::{ConfigSecret, SedeSecret};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Generate a Secrets_secret.toml from a list of sede config files.
struct Args {
    /// Sede config files (TOML with a [titulo] table and optional [[sedes]]).
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Write the generated secrets to this file instead of stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn secret() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 12)
}

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    // Collect unique sede names in first-seen order (titulo first, then sedes).
    let mut names: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for path in &args.files {
        let f = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("reading {}", path.display()))?;
        let contest: ConfigContest = toml::from_str(&f)
            .wrap_err_with(|| format!("parsing {}", path.display()))?;
        for name in std::iter::once(contest.titulo.name)
            .chain(contest.sedes.into_iter().flatten().map(|s| s.name))
        {
            if seen.insert(name.clone()) {
                names.push(name);
            }
        }
    }

    // Generate one unique 12-char alphanumeric secret per name.
    let mut used: HashSet<String> = HashSet::new();
    let mut secrets: Vec<SedeSecret> = Vec::new();
    for name in names {
        let mut s = secret();
        while !used.insert(s.clone()) {
            s = secret();
        }
        secrets.push(SedeSecret { name, secret: s });
    }
    let count = secrets.len();

    let out = toml::to_string(&ConfigSecret { secrets })?;
    match &args.output {
        Some(path) => {
            std::fs::write(path, &out)
                .wrap_err_with(|| format!("writing {}", path.display()))?;
            eprintln!("wrote {} secrets to {}", count, path.display());
        }
        None => print!("{out}"),
    }

    Ok(())
}
