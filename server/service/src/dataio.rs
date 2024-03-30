use crate::errors::{Error, ServiceResult};
use data::*;
use html_escape::decode_html_entities_to_string;
use std::fs::File;
use std::io::{self, Read};

pub trait FromString {
    fn from_string(s: &str) -> ServiceResult<Self>
    where
        Self: std::marker::Sized;
}

pub trait FromFile {
    fn from_file(s: &str) -> ServiceResult<Self>
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
        Ok(Team::new(team_line[0], team_line[1], team_name))
    }
}

fn read_to_string(s: &str) -> io::Result<String> {
    let mut file = File::open(s)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}

fn from_string_answer(t: &str, tim: i64) -> ServiceResult<Answer> {
    match t {
        "Y" => Ok(Answer::Yes(tim)),
        "N" => Ok(Answer::No),
        "X" => Ok(Answer::Unk),
        "?" => Ok(Answer::Wait),
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
        let ans = from_string_answer(v[4], time)?;

        Ok(Self {
            id,
            time,
            team_login: v[2].to_string(),
            prob: v[3].to_string(),
            answer: ans,
        })
    }
}

impl FromString for ContestFile {
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

        Ok(Self::new(
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

impl FromFile for ContestFile {
    fn from_file(s: &str) -> ServiceResult<Self> {
        let s = read_to_string(s)?;
        Self::from_string(&s)
    }
}

impl FromFile for RunsFile {
    fn from_file(s: &str) -> ServiceResult<Self> {
        let s = read_to_string(s)?;
        Self::from_string(&s)
    }
}

impl FromString for RunsFile {
    fn from_string(s: &str) -> ServiceResult<Self> {
        let runs = s.lines().map(RunTuple::from_string);
        let runs = runs.collect::<ServiceResult<_>>()?;
        Ok(RunsFile::new(runs))
    }
}

#[derive(Debug)]
pub struct DB {
    run_file: RunsFile,
    pub run_file_secret: RunsFile,
    pub contest_file_begin: ContestFile,
    contest_file: ContestFile,
    pub time_file: TimeFile,
}

pub fn read_contest(s: &str) -> ServiceResult<ContestFile> {
    ContestFile::from_string(s)
}

pub fn read_runs(s: &str) -> ServiceResult<RunsFile> {
    RunsFile::from_string(s)
}

impl DB {
    pub fn latest(&self) -> Vec<RunsPanelItem> {
        self.run_file
            .sorted()
            .into_iter()
            .filter(|r| r.time < self.contest_file.score_freeze_time)
            .map(|r| {
                let dummy = Team::dummy();
                let t = self.contest_file.teams.get(&r.team_login).unwrap_or(&dummy);
                RunsPanelItem {
                    id: r.id,
                    placement: t.placement,
                    color: 0,
                    escola: t.escola.clone(),
                    team_name: t.name.clone(),
                    team_login: t.login.clone(),
                    problem: r.prob.clone(),
                    result: r.answer.clone(),
                }
            })
            .collect()
    }

    pub fn empty() -> Self {
        DB {
            run_file: RunsFile::empty(),
            run_file_secret: RunsFile::empty(),
            contest_file_begin: ContestFile::dummy(),
            contest_file: ContestFile::dummy(),
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

#[cfg(test)]
mod tests {

    use super::*;
    use data::revelation::RevelationDriver;

    #[test]
    fn test_from_string() -> ServiceResult<()> {
        let x = "375971416299teambrbr3BN";
        let t = RunTuple::from_string(x)?;

        assert_eq!(t.id, 375971416);
        assert_eq!(t.time, 299);
        assert_eq!(t.team_login, "teambrbr3");
        assert_eq!(t.prob, "B");
        assert_eq!(t.answer, Answer::No);
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
    fn test_db_file_1a_fase_2020() -> ServiceResult<()> {
        let runs = RunsFile::from_file("test/webcast_zip_1a_fase_2020/runs")?;
        let contest = ContestFile::from_file("test/webcast_zip_1a_fase_2020/contest")?;
        assert_eq!(runs.len(), 6285);

        let mut db = DB::empty();
        db.refresh_db(0, contest, runs)?;

        assert_eq!(db.run_file.len(), 4927);
        assert_eq!(db.run_file_secret.len(), 6285);

        Ok(())
    }

    #[test]
    fn test_revelation_1a_fase_2020() -> ServiceResult<()> {
        let contest = ContestFile::from_file("test/webcast_zip_1a_fase_2020/contest")?;

        let runs = RunsFile::from_file("test/webcast_zip_1a_fase_2020/runs")?;
        assert_eq!(runs.len(), 6285);

        let r1 = RevelationDriver::new(contest.clone(), runs.clone())?;
        let r2 = RevelationDriver::new(contest, runs)?;

        for t in r1.contest().teams.values() {
            let t2_p = r2.contest().placement(&t.login).unwrap();
            assert_eq!(t.placement, t2_p);
        }

        for t in r2.contest().teams.values() {
            let t1_p = r1.contest().placement(&t.login).unwrap();
            assert_eq!(t.placement, t1_p);
        }

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
