//! Conversions from the public-API wire types to the legacy client types.
//!
//! The client model still consumes the legacy shapes; this bridge lets the
//! SDK talk to the new public API without touching `client-model`.

use data::configdata::{ConfigContest, RegexSetField, SedeEntry};
use data::event::{PublicConfig, PublicContestState, PublicTimer, Run, RunsData};
use data::{Answer as LegacyAnswer, ContestFile, RunTuple, RunsFile, Team, TimerData};

fn to_team(info: &data::event::TeamInfo) -> Team {
    Team {
        login: info.login.clone(),
        escola: info.escola.clone(),
        name: info.nome.clone(),
        placement: 0,
        placement_global: 0,
        problems: Default::default(),
        id: data::gen_id(),
    }
}

pub fn to_contest_file(state: PublicContestState) -> ContestFile {
    ContestFile {
        contest_name: state.contest,
        teams: state
            .teams
            .iter()
            .map(|team| (team.login.clone(), to_team(team)))
            .collect(),
        current_time: state.time_seconds,
        maximum_time: 0,
        score_freeze_time: state.score_freeze_time_seconds,
        penalty_per_wrong_answer: state.penalty_seconds,
        number_problems: state.problems.as_ref().map(Vec::len).unwrap_or(0),
    }
}

fn to_sede_entry(
    name: &str,
    codes: &[String],
    style: Option<&str>,
    medals: (usize, usize, usize),
) -> SedeEntry {
    let codes: RegexSetField =
        serde_json::from_value(serde_json::json!(codes)).unwrap_or_default();
    SedeEntry {
        name: name.to_string(),
        codes,
        style: style.map(str::to_string),
        ouro: medals.0,
        prata: medals.1,
        bronze: medals.2,
    }
}

pub fn to_config_contest(config: PublicConfig) -> ConfigContest {
    let sedes: Vec<SedeEntry> = config
        .sites
        .iter()
        .map(|site| to_sede_entry(&site.name, &site.codes, None, (1, 2, 3)))
        .collect();
    ConfigContest {
        titulo: to_sede_entry(
            &config.name,
            &config.codes,
            config.style.as_deref(),
            (config.ouro, config.prata, config.bronze),
        ),
        sedes: Some(sedes),
    }
}

pub fn to_timer_data(timer: PublicTimer) -> TimerData {
    TimerData {
        current_time: timer.current_time_seconds,
        score_freeze_time: timer.score_freeze_time_seconds,
    }
}

pub fn to_run_tuple(run: &Run, order: u64) -> RunTuple {
    let prob = run
        .prob
        .parse()
        .unwrap_or_else(|_| "A".parse().expect("A is a valid letter"));
    let answer = match run.answer {
        data::event::Answer::Yes => LegacyAnswer::Yes {
            time: run.time_seconds,
            is_first: false,
            run_id: run.id,
        },
        data::event::Answer::No => LegacyAnswer::No { run_id: run.id },
        data::event::Answer::Unknown => LegacyAnswer::Wait { run_id: run.id },
        data::event::Answer::Halt => LegacyAnswer::Unk { run_id: run.id },
    };
    RunTuple {
        id: run.id,
        order,
        time: run.time_seconds,
        team_login: run.team_login.clone(),
        prob,
        answer,
    }
}

pub fn to_runs_file(data: RunsData) -> RunsFile {
    RunsFile::from_runs(
        data.runs
            .iter()
            .enumerate()
            .map(|(order, run)| to_run_tuple(run, order as u64))
            .collect(),
    )
}
