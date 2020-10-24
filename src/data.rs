use std::io::{self, Read};
use std::fs::File;
use std::{error::Error, fmt};
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json;


fn read_to_string(s : &str) -> io::Result<String> {
    let mut file = File::open(s)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}
        

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
    InvalidUri(warp::http::uri::InvalidUri),
    Hyper(hyper::Error),
    Simple(String),
    UnmatchedTeam(String)
}

impl Error for ContestError {}

impl std::convert::From<std::num::ParseIntError> for ContestError {
    fn from(error: std::num::ParseIntError) -> Self {
        ContestError::Parse(error)
    }
}
impl std::convert::From<io::Error> for ContestError {
    fn from(error: io::Error) -> Self {
        ContestError::IO(error)
    }
}

impl std::convert::From<hyper::Error> for ContestError {
    fn from(error: hyper::Error) -> Self {
        ContestError::Hyper(error)
    }

}

impl std::convert::From<warp::http::uri::InvalidUri> for ContestError {
    fn from(error: warp::http::uri::InvalidUri) -> Self {
        ContestError::InvalidUri(error)
    }
}

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

#[derive(Debug, Serialize)]
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
                self.penalty += tim;
            },
            Answer::No => {
                // TODO many corner cases!
                self.submissions += 1;
                self.penalty += 20;
            },
            _ => {

            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Team {
    pub login : String,
    pub escola : String,
    pub name : String,
    pub placement : usize,
    pub problems : BTreeMap<String, Problem>,
}

// pub struct TeamScore {
//     pub login : String,
//     pub placement : String,
//     pub escola : String,
//     pub name : String,
//     pub solved_problems : usize,
//     pub penalty : usize,
//     pub solved : Vec<Option<Problem>>
// }

impl Team {
    fn new(login : &str, escola : &str, name : &str) -> Self {
        Self {
            login : login.to_string(),
            escola : escola.to_string(),
            name : name.to_string(),
            placement : 0,
            problems : BTreeMap::new()
        }
    }

    fn dummy() -> Self {
        Self::new("<login>", "<escola>", "<nome>")
    }

    fn from_contest_string(s : &str) -> Self {
        let team_line : Vec<_> = s.split("").collect();
        Team::new(team_line[0], team_line[1], team_line[2])
    }

    fn apply_run(&mut self, run : &RunTuple) {
        self.problems.entry(run.prob.clone())
            .or_insert(Problem::empty())
            .add_run_problem(run.time, run.answer.clone());
    }

    fn score(&self) -> (i64, i64) {
        let mut solved = 0;
        let mut penalty = 0;
        for (_, value) in self.problems.iter() {
            if value.solved {
                solved += 1;
                penalty += value.penalty;
            }
        }
        (solved, penalty)
    }
}

#[derive(Debug)]
pub struct ContestFile {
    pub contest_name : String,
    pub teams : BTreeMap<String, Team>,
    pub current_time : i64,
    pub maximum_time : i64,
    pub score_freeze_time : i64,
    pub penalty_per_wrong_answer : i64,
    pub score_board : Vec<String>
}

impl ContestFile {
    pub fn new(contest_name : String
        , teams : Vec<Team>
        , current_time : i64
        , maximum_time : i64
        , score_freeze_time : i64
        , penalty : i64 ) -> Self {

        let mut m = BTreeMap::new();
        for t in teams {
            m.insert(t.login.clone(), t);
        }
        Self {
            contest_name : contest_name,
            teams : m,
            current_time : current_time,
            maximum_time : maximum_time,
            score_freeze_time : score_freeze_time,
            penalty_per_wrong_answer : penalty,
            score_board : Vec::new()
        }
    }
    pub fn from_file(s :&str) -> Result<Self, ContestError> {
        let s = read_to_string(s)?;
        Self::from_string(s)
    }

    pub fn reload_score(&mut self) -> Result<(), ContestError> {
        let mut score_board = Vec::new();
        for (key, _) in self.teams.iter() {
            score_board.push(key.clone());
        }
        score_board.sort_by(|a,b| {
            let (solved_a, penalty_a) = self.teams.get(a).unwrap().score();
            let (solved_b, penalty_b) = self.teams.get(b).unwrap().score();

            if solved_a == solved_b {
                return penalty_a.cmp(&penalty_b);
            }
            return solved_b.cmp(&solved_a);
        });
        
        for (i, v) in score_board.iter().enumerate() {
            match self.teams.get_mut(v) {
                None => return Err(ContestError::UnmatchedTeam(v.clone())),
                Some(t) => t.placement = i+1
            }
        }

        self.score_board = score_board;
        Ok(())
    }

    pub fn from_string(s : String) -> Result<Self, ContestError> {
        let mut lines = s.lines();

        let contest_name = lines.next().unwrap();
        let contest_params : Vec<&str> = lines.next().unwrap().split("").collect();
        let maximum_time = contest_params[0].parse()?;
        let current_time = contest_params[1].parse()?;
        let score_freeze_time = contest_params[2].parse()?;
        let penalty = contest_params[3].parse()?;

        let team_params : Vec<&str> = lines.next().unwrap().split("").collect();
        let number_teams : usize = team_params[0].parse()?;
        let _number_problems : usize = team_params[1].parse()?;
        
        let mut teams = Vec::new();
        for _ in 0..number_teams {
            let t = Team::from_contest_string(lines.next().unwrap());
            teams.push(t);
        }

        // let _line_unknown1 = lines.next();
        // let _line_unknown2 = lines.next();

        // for i in 0..number_teams {

        //     let line : Vec<_> = lines.next().unwrap().split(",")
        //         .map( |x| x.parse::<i64>() )
        //         .collect::<Result<_,_>>()?;

        //     // 330505,1,11,1379,290
        //     let _unk_1 = line[0];
        //     let placement = line[1];
        //     teams[i].placement = placement;
        //     let _num_solved_problems = line[2];
        //     let _unk_2 = line[3];
        //     let _unk_3 = line[4];
        // }

        Ok(Self::new(
            contest_name.to_string(),
            teams,
            current_time,
            maximum_time,
            score_freeze_time,
            penalty
        ))
    }

    pub fn dummy() -> Self {
        Self::new("Dummy Contest".to_string(), Vec::new(), 0, 0, 0, 0)
    }

    // pub fn add_run(&mut self, run : RunTuple) {
    //     match self.teams.get_mut(&run.team_login) {
    //         None => {

    //         },
    //         Some(t) => {
    //             t.problems.entry(run.prob)
    //                     .or_insert(Problem::empty())
    //                     .add_run_problem(run.time, run.answer)
    //         }
    //     }
    // }
}

#[derive(Debug, Clone)]
pub struct RunTuple {
    id : i64,
    pub time : i64,
    pub team_login : String,
    pub prob : String,
    pub answer : Answer
}

#[derive(Debug)] 
pub struct RunsFile {
    runs : Vec<RunTuple>
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

    pub fn reload_runs(&mut self, s: String) -> Result<(), ContestError> {
        let runs = RunsFile::from_string(s)?;
        self.run_file = runs;
        Ok(())
    }

    pub fn reload_contest(&mut self, s: String) -> Result<(), ContestError> {
        self.contest_file = ContestFile::from_string(s)?;
        Ok(())
    }

    pub fn reload_time(&mut self, s: String) -> Result<(), ContestError> {
        let t = s.parse()?;
        self.time_file = t;
        Ok(())
    }

    pub fn recalculate_score(&mut self)
     -> Result<(), ContestError> {
        for r in &self.run_file.runs {
            match self.contest_file.teams.get_mut(&r.team_login) {
                None => return Err(ContestError::Simple("Could not apply run to team".to_string())),
                Some(t) => t.apply_run(&r),
            }
        }        
        self.contest_file.reload_score()?;
        Ok(())
    }

    pub fn get_scoreboard(&self) -> (&Vec<String>, &BTreeMap<String, Team>) {
        (&self.contest_file.score_board, &self.contest_file.teams)
    }
}

#[derive(Debug, Serialize)]
pub struct RunsPanelItem {
    id : i64,
    placement : usize,
    color : i64,
    escola : String,
    team_name : String,
    problem : String,
    result : Answer
}

impl RunsFile {
    pub fn empty() -> Self {
        RunsFile {
            runs : Vec::new()
        }
    }

    pub fn from_file(s : &str) -> Result<Self, ContestError> {
        let s = read_to_string(s)?;
        Self::from_string(s)
    }
    
    pub fn from_string(s: String) -> Result<Self, ContestError> {
        let runs = s.lines()
            .map( |line| RunTuple::from_string(line) );
        let runs = runs.collect::<Result<_, _>>()?;
        Ok(RunsFile {
            runs: runs
        })
    }


    pub fn latest_n(&self, n : usize) -> Vec<RunTuple> {
        let mut ret = self.runs.clone();
        ret.sort_by(|a, b| 
            b.time.cmp(&a.time)
            // a.time.cmp(&b.time)
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
        let x = RunsFile::from_file("test/sample/runs")?;
        assert_eq!(x.runs.len(), 716);
        Ok(())
    }

    #[test]
    fn test_parse_contest_file() -> Result<(), ContestError> {
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