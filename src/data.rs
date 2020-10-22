use std::io::{self, Read};
use std::fs::File;
use std::{error::Error, fmt};
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json;


#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Answer {
    Yes,
    No,
    Wait,
    Unk
}

#[derive(Debug)]
pub enum ContestError {
    IO(io::Error),
    Parse(std::num::ParseIntError),
    Simple(String)
}

impl Error for ContestError {}

impl fmt::Display for ContestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Answer could not be parsed: {:?}", self)
    }
}


impl Answer {
    fn from_string(t : &str) -> Result<Answer, ContestError> {
        match t {
            "Y" => Ok(Self::Yes),
            "N" => Ok(Self::No),
            "?" => Ok(Self::Wait),
            _ => Err(ContestError::Simple(t.to_string()))
        }        
    }
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Answer::Yes => "Accepted",
            Answer::No => "Wrong Answer",
            Answer::Wait => "Judging...",
            _ => "Error!"
        })
    }
}

#[derive(Debug)]
pub struct Problem {
    solved : bool,
    submissions : i64,
    penalty: i64
}

impl Problem {
    fn empty() -> Self {
        Problem { solved : false, submissions : 0, penalty : 0 }
    }
    fn add_run_problem(&mut self, tim : i64, answer: Answer) {
        match answer {
            Answer::Yes => {
                self.solved = true;
                self.submissions += 1;
            },
            Answer::No => {
                // TODO many corner cases!
                self.submissions += 1;
                self.penalty += tim
            },
            _ => {

            }
        }
    }
}

#[derive(Debug)]
pub struct Team {
    pub login : String,
    pub escola : String,
    pub name : String,
    pub problems : BTreeMap<String, Problem>
}

#[derive(Debug)]
pub struct Contest {
    pub teams : BTreeMap<String, Team>,
    pub current_time : i64,
    pub maximum_time : i64,
    pub score_freeze_time : i64,
    pub penalty_per_wrong_answer : i64
}

impl Contest {
    pub fn new(teams : Vec<Team>
        , current_time : i64
        , maximum_time : i64
        , score_freeze_time : i64
        , penalty : i64 ) -> Self {

        let mut m = BTreeMap::new();
        for t in teams {
            m.insert(t.login.clone(), t);
        }
        Contest {
            teams : m,
            current_time : current_time,
            maximum_time : maximum_time,
            score_freeze_time : score_freeze_time,
            penalty_per_wrong_answer : penalty
        }
    }

    pub fn add_run(&mut self, run : RunTuple) {
        match self.teams.get_mut(&run.team_login) {
            None => {

            },
            Some(t) => {
                t.problems.entry(run.prob)
                        .or_insert(Problem::empty())
                        .add_run_problem(run.time, run.answer)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RunTuple {
    id : i64,
    pub time : i64,
    pub team_login : String,
    pub prob : String,
    pub answer : Answer
}

#[derive(Debug)] 
pub struct RunsPanel {
    runs : Vec<RunTuple>
}

impl RunsPanel {
    pub fn empty() -> Self {
        RunsPanel {
            runs : Vec::new()
        }
    }

    pub fn from_file(s : &str) -> Result<Self, ContestError> {
        let r = RunTuple::from_file(s)?;
        Ok(RunsPanel { runs : r })
    }

    pub fn latest_n(&self, n : usize) -> Vec<RunTuple> {
        let mut ret = self.runs.clone();
        ret.sort_by(|a, b| 
            a.time.cmp(&b.time)
        );
        ret.truncate(n);
        ret
    }

    pub fn add_run(&mut self, t : &RunTuple) {
        self.runs.push(t.clone())
    }
}

impl RunTuple {
    pub fn from_string(line : &str) -> Result<Self, ContestError> {
        let v : Vec<&str> = line.split('').collect();
        let id = v[0].parse().map_err(|e| ContestError::Parse(e))?;
        let time = v[1].parse().map_err(|e| ContestError::Parse(e))?;
        let ans = Answer::from_string(v[4])?;
        
        Ok(Self {
            id   : id,
            time : time,
            team_login : v[2].to_string(),
            prob : v[3].to_string(),
            answer : ans
        })
    }
    
    fn from_file(s: &str) -> Result<Vec<Self>, ContestError> {
        let mut file = File::open(s).map_err(|e| ContestError::IO(e))?;
        let mut s = String::new();
        
        file.read_to_string(&mut s).map_err(|e| ContestError::IO(e))?;
        
        s.lines().map( |line| Self::from_string(line) ).collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from_string() -> Result<(), ContestError> {
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
    fn test_parse_file() -> Result<(), ContestError> {
        let x = RunTuple::from_file("test/sample/runs")?;
        assert_eq!(x.len(), 716);
        Ok(())
    }
}