use super::args::*;
use color_eyre::eyre::{Context, Result, ensure, eyre};
use data::{event::*, incremental::*};
use reqwest::Method;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{io::Read, path::Path};

#[derive(Debug)]
pub struct RequestPlan {
    pub method: Method,
    pub segments: Vec<String>,
    pub body: Option<Value>,
    pub keep_runs: bool,
    pub metrics: bool,
    pub projection: Option<&'static str>,
}
fn request(method: Method, segments: Vec<String>, body: Option<Value>) -> RequestPlan {
    RequestPlan {
        method,
        segments,
        body,
        keep_runs: false,
        metrics: false,
        projection: None,
    }
}
fn event(id: EventId) -> Vec<String> {
    vec!["internal".into(), "events".into(), id.event]
}
fn contest(id: ContestId) -> Vec<String> {
    vec!["internal".into(), "contests".into(), id.event, id.contest]
}
fn site(id: SiteId) -> Vec<String> {
    vec![
        "internal".into(),
        "sites".into(),
        id.event,
        id.contest,
        id.site,
    ]
}
fn team(id: TeamId) -> Vec<String> {
    vec![
        "internal".into(),
        "events".into(),
        id.event,
        "teams".into(),
        id.login,
    ]
}
fn append(mut path: Vec<String>, value: &str) -> Vec<String> {
    path.push(value.into());
    path
}
fn read_json(path: &Path) -> Result<Value> {
    let raw = if path == Path::new("-") {
        let mut raw = String::new();
        std::io::stdin().read_to_string(&mut raw)?;
        raw
    } else {
        std::fs::read_to_string(path).wrap_err_with(|| format!("reading {}", path.display()))?
    };
    serde_json::from_str(&raw)
        .map_err(|_| eyre!("invalid JSON input (check syntax; requests must not use an envelope)"))
}
fn input(input: Input, flags: impl Serialize, teams_file: Option<&Path>) -> Result<Value> {
    let mut flags = serde_json::to_value(flags)?;
    if let Some(file) = input.file {
        ensure!(
            flags.as_object().is_some_and(|v| v.is_empty()) && teams_file.is_none(),
            "--file cannot be combined with field flags"
        );
        return read_json(&file);
    }
    if let Some(file) = teams_file {
        flags["teams"] = read_json(file)?;
    }
    Ok(flags)
}
fn fields_only(value: &Value, allowed: &[&str]) -> Result<()> {
    let object = value
        .as_object()
        .ok_or_else(|| eyre!("expected a JSON object"))?;
    for key in object.keys() {
        ensure!(allowed.contains(&key.as_str()), "unknown field: {key}");
    }
    Ok(())
}
fn decode<T: DeserializeOwned>(value: &Value) -> Result<T> {
    serde_json::from_value(value.clone()).map_err(|e| eyre!("invalid resource input: {e}"))
}
fn validate_codes(value: &Value) -> Result<()> {
    if let Some(codes) = value.get("codes") {
        let codes: Vec<String> = decode(codes)?;
        regex::RegexSet::new(codes).map_err(|_| eyre!("invalid regex in codes"))?;
    }
    Ok(())
}
fn resource(
    mut value: Value,
    name: &str,
    kind: &str,
    update: bool,
    clear: ClearFields,
) -> Result<Value> {
    let allowed: &[&str] = match kind {
        "event" => &[
            "name",
            "problems",
            "teams",
            "score_freeze_time_seconds",
            "penalty_seconds",
            "time_seconds",
            "salt",
        ],
        "contest" => &[
            "name",
            "codes",
            "salt",
            "style",
            "ouro",
            "prata",
            "bronze",
            "photo_url_format",
            "sound_url_format",
        ],
        "site" => &["name", "codes", "salt"],
        _ => unreachable!(),
    };
    fields_only(&value, allowed)?;
    for field in clear.unset {
        ensure!(
            allowed.contains(&field.as_str())
                && ["salt", "style", "photo_url_format", "sound_url_format"]
                    .contains(&field.as_str()),
            "field cannot be cleared: {field}"
        );
        ensure!(
            value.get(&field).is_none(),
            "cannot both set and unset {field}"
        );
        value[&field] = Value::Null;
    }
    if let Some(body_name) = value.get("name") {
        ensure!(
            body_name.as_str() == Some(name),
            "body name must equal the positional resource name"
        );
    }
    if !update {
        value["name"] = json!(name);
    }
    if let Some(teams) = value.get("teams").and_then(Value::as_array) {
        for team in teams {
            fields_only(team, &["login", "escola", "nome"])?;
        }
    }
    validate_codes(&value)?;
    if update {
        ensure!(
            !value.as_object().unwrap().is_empty(),
            "update requires at least one field"
        );
        match kind {
            "event" => {
                decode::<EventPatch>(&value)?;
            }
            "contest" => {
                decode::<ContestPatch>(&value)?;
            }
            _ => {
                decode::<SitePatch>(&value)?;
            }
        }
    } else {
        match kind {
            "event" => {
                decode::<EventState>(&value)?;
            }
            "contest" => {
                decode::<ContestConfig>(&value)?;
            }
            _ => {
                decode::<SiteConfig>(&value)?;
            }
        }
    }
    Ok(value)
}
fn codes(flags: CodeFlags) -> Result<Value> {
    ensure!(
        !flags.add.is_empty() || !flags.remove.is_empty(),
        "supply --add or --remove"
    );
    ensure!(
        !flags.add.iter().any(|p| flags.remove.contains(p)),
        "a pattern cannot be both added and removed"
    );
    regex::RegexSet::new(&flags.add).map_err(|_| eyre!("invalid regex in --add"))?;
    Ok(json!({"add":flags.add,"remove":flags.remove}))
}

pub fn plan(command: Command) -> Result<RequestPlan> {
    let plan = match command {
        Command::Events(command) => match command {
            EventCommand::List => {
                request(Method::GET, vec!["internal".into(), "events".into()], None)
            }
            EventCommand::Get(id) => request(Method::GET, event(id), None),
            EventCommand::Delete(id) => request(Method::DELETE, event(id), None),
            EventCommand::Salt { id, salt } => request(
                Method::POST,
                append(event(id), "salt"),
                Some(json!({"salt":salt})),
            ),
            EventCommand::Create {
                id,
                input: source,
                fields,
            } => {
                let teams_file = fields.teams_file.clone();
                let body = input(source, fields, teams_file.as_deref())?;
                let body = resource(body, &id.event, "event", false, ClearFields::default())?;
                request(Method::POST, event(id), Some(body))
            }
            EventCommand::Update {
                id,
                input: source,
                fields,
                clear,
                keep_runs,
            } => {
                ensure!(
                    source.file.is_none() || clear.unset.is_empty(),
                    "--file cannot be combined with --unset"
                );
                let teams_file = fields.teams_file.clone();
                let body = input(source, fields, teams_file.as_deref())?;
                let body = resource(body, &id.event, "event", true, clear)?;
                ensure!(
                    !keep_runs || body.get("teams").is_some(),
                    "--keep-runs requires a teams replacement"
                );
                let mut result = request(Method::PATCH, event(id), Some(body));
                result.keep_runs = keep_runs;
                result
            }
            EventCommand::Replace { id, input: source } => {
                let body = resource(
                    read_json(&source.file)?,
                    &id.event,
                    "event",
                    false,
                    ClearFields::default(),
                )?;
                request(Method::PUT, event(id), Some(body))
            }
        },
        Command::Contests(command) => match command {
            ContestCommand::List(id) => request(Method::GET, append(event(id), "contests"), None),
            ContestCommand::Get(id) => request(Method::GET, contest(id), None),
            ContestCommand::Delete(id) => request(Method::DELETE, contest(id), None),
            ContestCommand::Salt { id, salt } => request(
                Method::POST,
                append(contest(id), "salt"),
                Some(json!({"salt":salt})),
            ),
            ContestCommand::Codes { id, codes: flags } => request(
                Method::PATCH,
                append(contest(id), "codes"),
                Some(codes(flags)?),
            ),
            ContestCommand::Create {
                id,
                input: source,
                fields,
            } => {
                let body = input(source, fields, None)?;
                let body = resource(body, &id.contest, "contest", false, ClearFields::default())?;
                request(Method::POST, contest(id), Some(body))
            }
            ContestCommand::Update {
                id,
                input: source,
                fields,
                clear,
            } => {
                ensure!(
                    source.file.is_none() || clear.unset.is_empty(),
                    "--file cannot be combined with --unset"
                );
                let body = input(source, fields, None)?;
                let body = resource(body, &id.contest, "contest", true, clear)?;
                request(Method::PATCH, contest(id), Some(body))
            }
            ContestCommand::Replace { id, input: source } => {
                let body = resource(
                    read_json(&source.file)?,
                    &id.contest,
                    "contest",
                    false,
                    ClearFields::default(),
                )?;
                request(Method::PUT, contest(id), Some(body))
            }
        },
        Command::Sites(command) => match command {
            SiteCommand::List(id) => request(
                Method::GET,
                vec![
                    "internal".into(),
                    "events".into(),
                    id.event,
                    "contests".into(),
                    id.contest,
                    "sites".into(),
                ],
                None,
            ),
            SiteCommand::Get(id) => request(Method::GET, site(id), None),
            SiteCommand::Delete(id) => request(Method::DELETE, site(id), None),
            SiteCommand::Salt { id, salt } => request(
                Method::POST,
                append(site(id), "salt"),
                Some(json!({"salt":salt})),
            ),
            SiteCommand::Codes { id, codes: flags } => request(
                Method::PATCH,
                append(site(id), "codes"),
                Some(codes(flags)?),
            ),
            SiteCommand::Create {
                id,
                input: source,
                fields,
            } => {
                let body = input(source, fields, None)?;
                let body = resource(body, &id.site, "site", false, ClearFields::default())?;
                request(Method::POST, site(id), Some(body))
            }
            SiteCommand::Update {
                id,
                input: source,
                fields,
                clear,
            } => {
                ensure!(
                    source.file.is_none() || clear.unset.is_empty(),
                    "--file cannot be combined with --unset"
                );
                let body = input(source, fields, None)?;
                let body = resource(body, &id.site, "site", true, clear)?;
                request(Method::PATCH, site(id), Some(body))
            }
            SiteCommand::Replace { id, input: source } => {
                let body = resource(
                    read_json(&source.file)?,
                    &id.site,
                    "site",
                    false,
                    ClearFields::default(),
                )?;
                request(Method::PUT, site(id), Some(body))
            }
        },
        Command::Teams(command) => match command {
            TeamCommand::List(id) => {
                let mut r = request(Method::GET, event(id), None);
                r.projection = Some("teams");
                r
            }
            TeamCommand::Get(id) => request(Method::GET, team(id), None),
            TeamCommand::Add {
                id,
                input: source,
                fields,
            } => {
                let value = input(source, fields, None)?;
                let team: NewTeam = decode(&value)?;
                ensure!(!team.login.is_empty(), "login must not be empty");
                request(Method::POST, append(event(id), "teams"), Some(value))
            }
            TeamCommand::Update {
                id,
                input: source,
                fields,
            } => {
                let value = input(source, fields, None)?;
                let patch: TeamPatch = decode(&value)?;
                ensure!(!patch.is_empty(), "update requires at least one field");
                request(Method::PATCH, team(id), Some(value))
            }
            TeamCommand::Delete { id, keep_runs } => {
                let mut r = request(Method::DELETE, team(id), None);
                r.keep_runs = keep_runs;
                r
            }
        },
        Command::Problems(command) => match command {
            ProblemCommand::List(id) => {
                let mut r = request(Method::GET, event(id), None);
                r.projection = Some("problems");
                r
            }
            ProblemCommand::Add {
                event: name,
                problem,
            } => {
                ensure!(!problem.is_empty(), "problem must not be empty");
                request(
                    Method::POST,
                    append(event(EventId { event: name }), "problems"),
                    Some(json!({"problem": problem})),
                )
            }
            ProblemCommand::Delete {
                event: name,
                problem,
            } => request(
                Method::DELETE,
                append(append(event(EventId { event: name }), "problems"), &problem),
                None,
            ),
        },
        Command::Timer(TimerCommand::Set {
            event: name,
            seconds,
        }) => request(
            Method::PATCH,
            append(event(EventId { event: name }), "time"),
            Some(json!({"time_seconds":seconds})),
        ),
        Command::Runs(command) => match command {
            RunCommand::Delete { event: name, id } => request(
                Method::DELETE,
                append(
                    append(event(EventId { event: name }), "runs"),
                    &id.to_string(),
                ),
                None,
            ),
            RunCommand::Clear(id) => request(Method::DELETE, append(event(id), "runs"), None),
            RunCommand::Add {
                event: name,
                id,
                team_login,
                problem,
                time_seconds,
                answer,
            } => request(
                Method::POST,
                append(event(EventId { event: name }), "runs"),
                Some(
                    json!({"runs":[{"id":id,"team_login":team_login,"prob":problem,"time_seconds":time_seconds,"answer":answer}]}),
                ),
            ),
            RunCommand::Import {
                event: name,
                input: source,
            } => {
                let value = read_json(&source.file)?;
                fields_only(&value, &["runs"])?;
                let _: RunsData = decode(&value)?;
                for run in value["runs"].as_array().unwrap() {
                    fields_only(run, &["id", "team_login", "prob", "time_seconds", "answer"])?;
                }
                request(
                    Method::POST,
                    append(event(EventId { event: name }), "runs"),
                    Some(value),
                )
            }
        },
        Command::RevelationUrls(id) => {
            request(Method::GET, append(event(id), "revelation_urls"), None)
        }
        Command::Metrics => {
            let mut r = request(Method::GET, vec!["internal".into(), "metrics".into()], None);
            r.metrics = true;
            r
        }
    };
    ensure!(
        plan.segments
            .iter()
            .all(|s| !s.is_empty() && s != "." && s != ".."),
        "resource identifiers must be nonempty and cannot be . or .."
    );
    Ok(plan)
}
