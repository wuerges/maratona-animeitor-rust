use std::io::{self, Read};
use std::fs::File;
use std::collections::BTreeMap;

use maratona_animeitor_rust::data::*;

type ContestIOResult<T> = Result<T, ContestIOError>;

impl std::error::Error for ContestIOError {}

#[derive(Debug)]
pub enum ContestIOError {
    IO(io::Error),
    InvalidUri(warp::http::uri::InvalidUri),
    Hyper(hyper::Error),
    ParseInt(std::num::ParseIntError),
    InvalidAnswer(String),
    Chain(ContestError),
    Info(String) 
}

impl std::fmt::Display for ContestIOError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ContestIOError: {:?}", self)
    }
}

impl std::convert::From<io::Error> for ContestIOError {
    fn from(error: io::Error) -> Self {
        ContestIOError::IO(error)
    }
}

impl std::convert::From<hyper::Error> for ContestIOError {
    fn from(error: hyper::Error) -> Self {
        ContestIOError::Hyper(error)
    }

}

impl std::convert::From<warp::http::uri::InvalidUri> for ContestIOError {
    fn from(error: warp::http::uri::InvalidUri) -> Self {
        ContestIOError::InvalidUri(error)
    }
}

impl std::convert::From<maratona_animeitor_rust::data::ContestError> for ContestIOError {
    fn from(error: maratona_animeitor_rust::data::ContestError) -> Self {
        ContestIOError::Chain(error)
    }
}

impl std::convert::From<std::num::ParseIntError> for ContestIOError {
    fn from(error: std::num::ParseIntError) -> Self {
        ContestIOError::ParseInt(error)
    }
}

trait FromString {
    fn from_string(s : &str) -> ContestIOResult<Self>
    where Self: std::marker::Sized;
}

trait FromFile {
    fn from_file(s :&str) -> ContestIOResult<Self>
    where Self: std::marker::Sized;
}

impl FromString for Team {
    fn from_string(s : &str) -> ContestIOResult<Self> {
        let team_line : Vec<_> = s.split("").collect();
        Ok(Team::new(team_line[0], team_line[1], team_line[2]))
    }
}

fn read_to_string(s : &str) -> io::Result<String> {
    let mut file = File::open(s)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}

impl FromString for  Answer {
    fn from_string(t : &str) -> Result<Answer, ContestIOError> {
        match t {
            "Y" => Ok(Self::Yes),
            "N" => Ok(Self::No),
            "?" => Ok(Self::Wait),
            _ => Err(ContestIOError::InvalidAnswer(t.to_string()))
        }        
    }
}

impl FromString for RunTuple {
    fn from_string(line : &str) -> Result<Self, ContestIOError> {
        let v : Vec<&str> = line.split('').collect();
        let id = v[0].parse()?;
        let time = v[1].parse()?;
        let ans = Answer::from_string(v[4])?;
        
        Ok(Self {
            id   : id,
            time : time,
            team_login : v[2].to_string(),
            prob : v[3].to_string(),
            answer : ans
        })
    }
}

impl FromString for ContestFile {

    fn from_string(s : &str) -> Result<Self, ContestIOError> {
        let mut lines = s.lines();

        let contest_name = lines.next().unwrap();
        let contest_params : Vec<&str> = lines.next().unwrap().split("").collect();
        let maximum_time = contest_params[0].parse()?;
        let current_time = contest_params[1].parse()?;
        let score_freeze_time = contest_params[2].parse()?;
        let penalty = contest_params[3].parse()?;

        let team_params : Vec<&str> = lines.next().unwrap().split("").collect();
        let number_teams : usize = team_params[0].parse()?;
        let number_problems : usize = team_params[1].parse()?;
        
        let mut teams = Vec::new();
        for _ in 0..number_teams {
            let t = Team::from_string(lines.next().unwrap())?;
            teams.push(t);
        }

        Ok(Self::new(
            contest_name.to_string(),
            teams,
            current_time,
            maximum_time,
            score_freeze_time,
            penalty,
            number_problems
        ))
    }
}

impl FromFile for ContestFile {
    fn from_file(s :&str) -> Result<Self, ContestIOError> {
        let s = read_to_string(s)?;
        Self::from_string(&s)
    }
}

impl FromFile for RunsFile {
    fn from_file(s : &str) -> Result<Self, ContestIOError> {
        let s = read_to_string(s)?;
        Self::from_string(&s)
    }
}

impl FromString for RunsFile {    
    fn from_string(s: &str) -> ContestIOResult<Self> {
        let runs = s.lines()
            .map( |line| RunTuple::from_string(line) );
        let runs = runs.collect::<Result<_, _>>()?;
        Ok(RunsFile {
            runs
        })
    }

}


#[derive(Debug)]
pub struct DB {
    run_file : RunsFile,
    contest_file : ContestFile,
    time_file : i64
}

impl DB {
    pub fn latest_n(&self, n : usize) -> Vec<RunsPanelItem> {
        self.run_file.latest_n(n).into_iter().map(|r| {
            let dummy = Team::dummy();
            let t = self.contest_file.teams.get(&r.team_login)
                        .unwrap_or(&dummy);
            RunsPanelItem {
                id : r.id,
                placement: t.placement,
                color : 0,
                escola : t.escola.clone(),
                team_name : t.name.clone(),
                team_login : t.login.clone(),
                problem : r.prob,
                result : r.answer
            }
        }).collect()
    }

    pub fn empty() -> Self {
        DB {
            run_file : RunsFile::empty(),
            contest_file  : ContestFile::dummy(),
            time_file : 0

        }
    }

    pub fn get_scoreboard(&self) -> (&Vec<String>, &BTreeMap<String, Team>, usize) {
        (&self.contest_file.score_board, &self.contest_file.teams, self.contest_file.number_problems)
    }

    pub fn reload_runs(&mut self, s: &str) -> Result<(), ContestIOError> {
        let runs = RunsFile::from_string(s)?;
        self.run_file = runs;
        Ok(())
    }

    pub fn reload_contest(&mut self, s: &str) -> Result<(), ContestIOError> {
        self.contest_file = ContestFile::from_string(s)?;
        Ok(())
    }

    pub fn reload_time(&mut self, s: String) -> Result<(), ContestIOError> {
        let t = s.parse()?;
        self.time_file = t;
        Ok(())
    }

    pub fn recalculate_score(&mut self)
     -> Result<(), ContestError> {
        for r in self.run_file.runs.iter().rev() {
            match self.contest_file.teams.get_mut(&r.team_login) {
                None => return Err(ContestError::UnmatchedTeam("Could not apply run to team".to_string())),
                Some(t) => t.apply_run(&r),
            }
        }        
        self.contest_file.reload_score()?;
        Ok(())
    }

}



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from_string() -> Result<(), ContestIOError> {
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
    fn test_parse_file() -> Result<(), ContestIOError> {
        let x = RunsFile::from_file("test/sample/runs")?;
        assert_eq!(x.runs.len(), 716);
        Ok(())
    }

    #[test]
    fn test_parse_contest_file() -> Result<(), ContestIOError> {
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
