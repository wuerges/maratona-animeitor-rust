use crate::errors::{Error, ServiceResult};
use data::configdata::Sede;
use data::*;
use html_escape::decode_html_entities_to_string;
use std::collections::BTreeMap;
use tracing::{Level, instrument};

pub trait FromString {
    fn from_string(s: &str) -> ServiceResult<Self>
    where
        Self: std::marker::Sized;
}

impl FromString for Team {
    fn from_string(s: &str) -> ServiceResult<Self> {
        let team_line: Vec<_> = s.split('').collect();
        if team_line.len() != 3 {
            return Err(Error::Parse("failed parsing Team".into()));
        }
        let mut team_name = String::new();
        decode_html_entities_to_string(team_line[2], &mut team_name);
        Ok(team_new(team_line[0], team_line[1], team_name))
    }
}

fn from_string_answer(t: &str, time: i64, run_id: i64) -> ServiceResult<Answer> {
    match t {
        "Y" => Ok(Answer::Yes {
            time,
            is_first: false,
            run_id,
        }),
        "N" => Ok(Answer::No { run_id }),
        "X" => Ok(Answer::Unk { run_id }),
        "?" => Ok(Answer::Wait { run_id }),
        _ => Err(Error::InvalidAnswer(t.to_string())),
    }
}

impl FromString for RunTuple {
    fn from_string(line: &str) -> ServiceResult<Self> {
        let v: Vec<&str> = line.split('').collect();
        if v.len() != 5 {
            return Err(Error::Parse("failed parsing RunTuple".into()));
        }
        let id = v[0].parse()?;
        let time = v[1].parse()?;
        let ans = from_string_answer(v[4], time, id)?;

        Ok(Self {
            order: 0,
            id,
            time,
            team_login: v[2].to_string(),
            prob: v[3].parse()?,
            answer: ans,
        })
    }
}

impl FromString for ContestFile {
    #[instrument(ret(level = Level::DEBUG), err)]
    fn from_string(s: &str) -> ServiceResult<Self> {
        let mut lines = s.lines();

        let contest_name = lines
            .next()
            .ok_or(Error::ContestFileParse("contest name"))?;
        let contest_params: Vec<&str> = lines
            .next()
            .ok_or(Error::ContestFileParse("timing params"))?
            .split('')
            .collect();

        if contest_params.len() != 4 {
            return Err(Error::Parse("failed parsing contest_params".into()));
        }

        let maximum_time = contest_params[0].parse()?;
        let current_time = contest_params[1].parse()?;
        let score_freeze_time = contest_params[2].parse()?;
        let penalty = contest_params[3].parse()?;

        let team_params: Vec<&str> = lines
            .next()
            .ok_or(Error::ContestFileParse("team params"))?
            .split('')
            .collect();

        if team_params.len() != 2 {
            return Err(Error::Parse("failed parsing contest_params".into()));
        }

        let number_teams: usize = team_params[0].parse()?;
        let number_problems: usize = team_params[1].parse()?;

        let mut teams = Vec::new();
        for _ in 0..number_teams {
            let t = Team::from_string(lines.next().ok_or(Error::ContestFileParse("team"))?)?;
            teams.push(t);
        }

        Ok(contest_file_new(
            contest_name.to_string(),
            teams,
            current_time,
            maximum_time,
            score_freeze_time,
            penalty,
            number_problems,
        ))
    }
}

impl FromString for RunsFile {
    fn from_string(s: &str) -> ServiceResult<Self> {
        let runs = read_runs(s)?;
        Ok(runs_file_new(runs))
    }
}

#[derive(Debug)]
pub struct DB {
    run_file: RunsFile,
    pub run_file_secret: RunsFile,
    pub contest_file_begin: ContestFile,
    pub time_file: TimeFile,
}

pub fn read_contest(s: &str) -> ServiceResult<ContestFile> {
    ContestFile::from_string(s)
}

pub fn read_runs(s: &str) -> ServiceResult<Vec<RunTuple>> {
    let runs = s.lines().map(RunTuple::from_string).rev();
    let mut runs = runs.collect::<ServiceResult<Vec<RunTuple>>>()?;

    for (i, run) in runs.iter_mut().enumerate() {
        run.order = i as u64;
    }

    Ok(runs)
}

impl DB {
    pub fn empty() -> Self {
        DB {
            run_file: RunsFile::empty(),
            run_file_secret: RunsFile::empty(),
            contest_file_begin: ContestFile {
                contest_name: "Dummy Contest".to_string(),
                teams: BTreeMap::new(),
                current_time: 0,
                maximum_time: 0,
                score_freeze_time: 0,
                penalty_per_wrong_answer: 0,
                number_problems: 0,
            },
            time_file: 0,
        }
    }

    pub fn refresh_db(
        &mut self,
        time: i64,
        contest: ContestFile,
        mut runs: RunsFile,
    ) -> ServiceResult<Vec<RunTuple>> {
        self.time_file = time;
        self.contest_file_begin = contest;

        runs.filter_teams(&self.contest_file_begin);
        let runs_frozen = runs.filter_frozen(self.contest_file_begin.score_freeze_time);

        let fresh = self.run_file.refresh(runs_frozen.sorted());
        self.run_file_secret = runs;

        Ok(fresh)
    }

    pub fn timer_data(&self) -> TimerData {
        TimerData::new(self.time_file, self.contest_file_begin.score_freeze_time)
    }

    pub fn all_runs(&self) -> Vec<RunTuple> {
        self.run_file.sorted()
    }
}

fn team_new(login: &str, escola: &str, name: String) -> Team {
    Team {
        login: login.to_string(),
        escola: escola.to_string(),
        name,
        placement: 0,
        placement_global: 0,
        problems: BTreeMap::new(),
        id: data::gen_id(),
    }
}

fn contest_file_new(
    contest_name: String,
    teams: Vec<Team>,
    current_time: i64,
    maximum_time: i64,
    score_freeze_time: i64,
    penalty: i64,
    number_problems: usize,
) -> ContestFile {
    let mut m = BTreeMap::new();
    for t in teams {
        m.insert(t.login.clone(), t);
    }
    ContestFile {
        contest_name,
        teams: m,
        current_time,
        maximum_time,
        score_freeze_time,
        penalty_per_wrong_answer: penalty,
        number_problems,
    }
}

pub fn runs_file_new(runs: Vec<RunTuple>) -> RunsFile {
    let mut t = RunsFile::empty();
    for r in runs {
        t.refresh_1(&r);
    }
    t
}

/// Server-side transforms of the shared [`RunsFile`] wire type.
pub trait RunsFileExt {
    /// Rewrites runs at or after `frozen_time` to `Answer::Wait`.
    fn filter_frozen(&self, frozen_time: i64) -> RunsFile;

    /// Keeps only the runs of teams belonging to the given site.
    fn filter_sede(&self, sede: &Sede) -> RunsFile;

    /// Inserts/updates the given runs, returning the ones that changed.
    fn refresh(&mut self, fresh: Vec<RunTuple>) -> Vec<RunTuple>;
}

impl RunsFileExt for RunsFile {
    fn filter_frozen(&self, frozen_time: i64) -> RunsFile {
        runs_file_new(
            self.sorted()
                .into_iter()
                .map(|mut r| {
                    if r.time >= frozen_time {
                        let run_id = match r.answer {
                            Answer::Yes { run_id, .. }
                            | Answer::No { run_id }
                            | Answer::Wait { run_id }
                            | Answer::Unk { run_id } => run_id,
                        };
                        r.answer = Answer::Wait { run_id };
                    }
                    r
                })
                .collect(),
        )
    }

    fn filter_sede(&self, sede: &Sede) -> RunsFile {
        let mut out = RunsFile::empty();
        for run in self.sorted() {
            if sede.team_belongs_str(&run.team_login) {
                out.refresh_1(&run);
            }
        }
        out
    }

    fn refresh(&mut self, fresh: Vec<RunTuple>) -> Vec<RunTuple> {
        let mut rec = Vec::new();

        for t in fresh {
            if self.refresh_1(&t) {
                rec.push(t);
            }
        }

        rec
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{self, Read};

    use super::*;

    trait FromFile {
        fn from_file(s: &str) -> ServiceResult<Self>
        where
            Self: std::marker::Sized;
    }

    impl<T: FromString> FromFile for T {
        fn from_file(s: &str) -> ServiceResult<Self> {
            let s = read_to_string(s)?;
            Self::from_string(&s)
        }
    }

    fn read_to_string(s: &str) -> io::Result<String> {
        let mut file = File::open(s)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Ok(s)
    }

    #[test]
    fn test_from_string() -> ServiceResult<()> {
        let x = "375971416299teambrbr3BN";
        let t = RunTuple::from_string(x)?;

        assert_eq!(t.id, 375971416);
        assert_eq!(t.time, 299);
        assert_eq!(t.team_login, "teambrbr3");
        assert_eq!(t.prob, "B".parse().unwrap());
        assert_eq!(t.answer, Answer::No { run_id: 375971416 });
        Ok(())
    }

    #[test]
    fn test_from_string_throws_error() {
        let x = "375971416";
        let t = RunTuple::from_string(x);
        assert!(
            t.is_err(),
            "parsing empty arrays should be an error: {:?}",
            t
        );
    }

    #[test]
    fn test_parse_file() -> ServiceResult<()> {
        let x = RunsFile::from_file("test/sample/runs")?;
        assert_eq!(x.len(), 716);
        Ok(())
    }

    #[test]
    fn test_parse_file_1a_fase_2020() -> ServiceResult<()> {
        let x = RunsFile::from_file("test/webcast_zip_1a_fase_2020/runs")?;
        assert_eq!(x.len(), 6285);
        Ok(())
    }

    #[test]
    fn test_parse_contest_file() -> ServiceResult<()> {
        let x = ContestFile::from_file("test/sample/contest")?;
        assert_eq!(x.contest_name, "LATAM ACM ICPC".to_string());
        assert_eq!(x.maximum_time, 300);
        assert_eq!(x.current_time, 285);
        assert_eq!(x.score_freeze_time, 240);
        assert_eq!(x.penalty_per_wrong_answer, 20);
        assert_eq!(x.teams.keys().len(), 72);
        Ok(())
    }
}
