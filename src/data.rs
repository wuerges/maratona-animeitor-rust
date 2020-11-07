use std::fmt;
use std::collections::{BTreeMap, BinaryHeap};
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Answer {
    Yes(usize),
    No,
    Wait,
    Unk
}

#[derive(Debug)]
pub enum ContestError {
    UnmatchedTeam(String)
}

impl std::error::Error for ContestError {}

impl fmt::Display for ContestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ContestError: {:?}", self)
    }
}


impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type TimeFile = i64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub solved : bool,
    pub submissions : usize,
    pub penalty: usize,
    pub time_solved : usize,
    pub answers: Vec<Answer>,
}

impl Problem {
    fn empty() -> Self {
        Problem { solved : false, submissions : 0, time_solved: 0, penalty : 0, answers: Vec::new() }
    }
    fn add_run_problem(&mut self, answer: Answer) {
        if self.solved {
            return;
        }
        match answer {
            Answer::Yes(tim) => {
                self.solved = true;
                self.submissions += 1;
                self.penalty += tim;
                self.time_solved = tim;
                self.answers.clear();
            },
            Answer::No => {
                self.submissions += 1;
                self.penalty += 20;
                self.answers.clear();
            },
            Answer::Wait => {
                self.answers.push(Answer::No) // failsafe
            },
            _ => {

            }
        }
    }

    pub fn wait(&self) -> bool {
        !self.solved && self.answers.len() > 0
    }

    fn add_run_frozen(&mut self, answer: Answer) {
        self.answers.push(answer)
    }

    fn reveal_run_frozen(&mut self) {
        if self.answers.len() > 0 {
            let a = self.answers.remove(0);
            self.add_run_problem(a);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub login : String,
    pub escola : String,
    pub name : String,
    pub placement : usize,
    pub problems : BTreeMap<String, Problem>,
}

use std::cmp::{Ord, Eq, Ordering};

#[derive(PartialEq, Eq)]
pub struct Score {
    pub solved: usize,
    pub penalty: usize,
    pub max_solution_time: usize,
    pub team_login: String,    
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(if self.solved != other.solved {
            other.solved.cmp(&self.solved)
        }
        else if self.penalty != other.penalty {
            self.penalty.cmp(&other.penalty)
        }
        else if self.max_solution_time != other.max_solution_time {
            self.max_solution_time.cmp(&other.max_solution_time)
        }
        else {
            self.team_login.cmp(&other.team_login)
        })
    }    

}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Team {
    pub fn new(login : &str, escola : &str, name : &str) -> Self {
        Self {
            login : login.to_string(),
            escola : escola.to_string(),
            name : name.to_string(),
            placement : 0,
            problems : BTreeMap::new()
        }
    }

    pub fn dummy() -> Self {
        Self::new("<login>", "<escola>", "<nome>")
    }


    fn apply_run(&mut self, run : &RunTuple) {
        self.problems.entry(run.prob.clone())
            .or_insert(Problem::empty())
            .add_run_problem(run.answer.clone());
    }

    fn apply_run_frozen(&mut self, run : &RunTuple) {
        self.problems.entry(run.prob.clone())
            .or_insert(Problem::empty())
            .add_run_frozen(run.answer.clone());
    }

    fn wait(&self) -> bool {
        // false
        self.problems.values()
        .map(|p| p.wait())
        .fold(false, |t,e| t || e)
    }

    fn reveal_run_frozen(&mut self) {
        for p in self.problems.values_mut() {
            if p.wait() {
                p.reveal_run_frozen();
                return;
            }
        }
    }


    // fn useful_run(&self, run : &RunTuple) -> bool {
    //     self.problems.get(&run.prob).map(|p| !p.solved ).unwrap_or(true)
    // }

    pub fn score(&self) -> Score {
        let mut solved = 0;
        let mut penalty = 0;
        let mut max_solution_time = 0;
        for (_, value) in self.problems.iter() {
            if value.solved {
                solved += 1;
                penalty += value.penalty;
                max_solution_time = max_solution_time.max(value.time_solved);
            }
        }
        Score { solved, penalty, max_solution_time, team_login: self.login.clone() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContestFile {
    pub contest_name : String,
    pub teams : BTreeMap<String, Team>,
    pub current_time : usize,
    pub maximum_time : usize,
    pub score_freeze_time : usize,
    pub penalty_per_wrong_answer : usize,
    pub score_board : Vec<String>,
    pub number_problems : usize
}

impl ContestFile {
    pub fn new(contest_name : String
        , teams : Vec<Team>
        , current_time : usize
        , maximum_time : usize
        , score_freeze_time : usize
        , penalty : usize
        , number_problems : usize) -> Self {

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
            score_board : Vec::new(),
            number_problems : number_problems
        }
    }

    pub fn placement(&self, team_login: &String) -> Option<usize> {
        self.teams.get(team_login).map(|t| t.placement)
    }

    pub fn recalculate_placement(&mut self) -> Result<(), ContestError> {
        let mut score_board = Vec::new();
        for (key, _) in self.teams.iter() {
            score_board.push(key.clone());
        }
        score_board.sort_by(|a,b| {
            let score_a = self.teams.get(a).unwrap().score();
            let score_b = self.teams.get(b).unwrap().score();
            score_a.cmp(&score_b)
        });
        
        for (i, v) in score_board.iter().enumerate() {
            match self.teams.get_mut(v) {
                None => return Err(ContestError::UnmatchedTeam(v.clone())),
                Some(t) => t.placement = i+1
            }
        }

        Ok(())
    }

    pub fn reload_score(&mut self) -> Result<(), ContestError> {
        let mut score_board = Vec::new();
        for (key, _) in self.teams.iter() {
            score_board.push(key.clone());
        }
        score_board.sort_by(|a,b| {
            let score_a = self.teams.get(a).unwrap().score();
            let score_b = self.teams.get(b).unwrap().score();
            score_a.cmp(&score_b)
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


    pub fn dummy() -> Self {
        Self::new("Dummy Contest".to_string(), Vec::new(), 0, 0, 0, 0, 0)
    }

    // pub fn useful_run(&self, r : &RunTuple) -> Result<bool, ContestError> {
    //     match self.teams.get(&r.team_login) {
    //         None => Err(ContestError::UnmatchedTeam(
    //             "Could not check useful run to team".to_string(),
    //         )),
    //         Some(t) => {                
    //             Ok(t.useful_run(r))
    //         }
    //     }   
    // }

    pub fn apply_run(&mut self, r : &RunTuple) -> Result<(), ContestError> {
        match self.teams.get_mut(&r.team_login) {
            None => Err(ContestError::UnmatchedTeam(
                "Could not apply run to team".to_string(),
            )),
            Some(t) => {
                t.apply_run(&r);
                Ok(())
            }
        }
    }

    pub fn apply_run_frozen(&mut self, r : &RunTuple) -> Result<Score, ContestError> {
        match self.teams.get_mut(&r.team_login) {
            None => Err(ContestError::UnmatchedTeam(
                "Could not apply run to team".to_string(),
            )),
            Some(t) => {
                t.apply_run_frozen(&r);
                Ok(t.score())
            }
        }
    }

    // pub fn mark_all_wrong(&mut self, team_name : &String) {
    //     match self.teams.get_mut(team_name) {
    //         None => (),
    //         Some(t) => t.mark_all_wrong(),
    //     }
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTuple {
    pub id : i64,
    pub time : usize,
    pub team_login : String,
    pub prob : String,
    pub answer : Answer
}

#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct RunsFile {
    pub runs : Vec<RunTuple>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunsPanelItem {
    pub id : i64,
    pub placement : usize,
    pub color : i64,
    pub escola : String,
    pub team_name : String,
    pub team_login : String,
    pub problem : String,
    pub result : Answer
}

impl RunsFile {
    pub fn empty() -> Self {
        RunsFile {
            runs : Vec::new()
        }
    }

    pub fn sorted(&self) -> Vec<RunTuple> {
        let mut ret = self.runs.clone();
        ret.sort_by(|a, b| 
            b.time.cmp(&a.time)
            // a.time.cmp(&b.time)
        );
        ret
    }

    pub fn add_run(&mut self, t : &RunTuple) {
        self.runs.push(t.clone())
    }
}

pub struct RunsQueue {
    pub queue : BinaryHeap<Score>,
}

impl RunsQueue {
    pub fn empty() -> Self {
        Self {
            queue : BinaryHeap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn setup_teams(&mut self, contest: &ContestFile) {
        for team in contest.teams.values() {
            if team.wait() {
                self.queue.push(team.score())
            }
        }
    }

    pub fn pop_run(&mut self, contest: &mut ContestFile) {

        let entry = self.queue.pop();
        match entry {
            None => (),
            Some(score) => {
                match contest.teams.get_mut(&score.team_login) {
                    None => (),
                    Some(team) => {
                        team.reveal_run_frozen();
                        if team.wait() {
                            self.queue.push(team.score());
                        }
                    }
                }
            }
        }
    }

}