use crate::configdata::*;
use crate::data::*;

use std::collections::{BTreeMap, BinaryHeap};

pub struct Revelation {
    pub contest: ContestFile,
    runs: RunsFile,
    runs_queue: RunsQueue,
}

pub enum Event {
    Dud,
    Winner {
        team_login: String,
        nome_sede: String,
    },
}

pub struct RevelationDriver {
    revelation: Revelation,
    // _sedes: ConfigContest,
    winners: BTreeMap<String, String>,
}

impl RevelationDriver {
    fn calculate_winners(
        contest: ContestFile,
        runs: RunsFile,
        sedes: &ConfigContest,
    ) -> BTreeMap<String, String> {
        let mut mock = Revelation::new(contest.clone(), runs.clone());
        mock.apply_all_runs();

        let mut teams: Vec<_> = mock.contest.teams.values().collect();
        teams.sort();
        let mut winners = BTreeMap::new();
        for t in teams {
            let sede = sedes.get_sede(&t.login).unwrap();
            winners.entry(sede).or_insert(t.login.clone());
        }

        let mut rev_winners = BTreeMap::new();
        for (k, v) in winners {
            rev_winners.entry(v).or_insert(k);
        }
        rev_winners
    }

    pub fn new(contest: ContestFile, runs: RunsFile, sedes: ConfigContest) -> Self {
        let winners = Self::calculate_winners(contest.clone(), runs.clone(), &sedes);

        let mut revelation = Revelation::new(contest, runs);
        revelation.apply_all_runs_before_frozen();

        Self {
            revelation,
            // sedes,
            winners,
        }
    }

    pub fn reveal_step(&mut self) -> Option<Event> {
        let team = self.revelation.apply_one_run_from_queue();
        let winners = &self.winners;
        team.map(|team| {
            match winners.get(&team.login) {
                None => Event::Dud,
                Some(sede) => Event::Winner { team_login : team.login.clone(), nome_sede : sede.clone() },
            }
        })
    }
}

impl Revelation {
    pub fn new(contest: ContestFile, runs: RunsFile) -> Self {
        Self {
            contest,
            runs,
            runs_queue: RunsQueue::empty(),
        }
    }

    pub fn apply_all_runs_before_frozen(&mut self) {
        for run in self.runs.sorted() {
            if run.time < self.contest.score_freeze_time {
                self.contest.apply_run(run).unwrap();
            } else {
                self.contest.apply_run_frozen(run).unwrap();
            }
        }
        self.runs_queue = RunsQueue::setup_queue(&self.contest);
        self.contest.recalculate_placement().unwrap();
    }

    pub fn apply_all_runs_on_frozen(&mut self) {
        for run in self.runs.sorted() {
            self.contest.apply_run_frozen(run).unwrap();
        }
        self.runs_queue = RunsQueue::setup_queue(&self.contest);
        self.contest.recalculate_placement().unwrap();
    }

    pub fn apply_one_run_from_queue(&mut self) -> Option<&Team> {
        self.runs_queue.pop_run(&mut self.contest)
    }

    pub fn apply_all_runs_from_queue(&mut self) {
        while self.runs_queue.queue.len() > 0 {
            self.apply_one_run_from_queue();
        }
        self.contest.recalculate_placement().unwrap();
    }

    pub fn apply_all_runs(&mut self) {
        for run in self.runs.sorted() {
            self.contest.apply_run(run).unwrap();
        }
        self.contest.recalculate_placement().unwrap();
    }
}

pub struct RunsQueue {
    queue: BinaryHeap<Score>,
}

impl RunsQueue {
    fn empty() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn setup_queue(contest: &ContestFile) -> Self {
        let mut q = Self::empty();
        for team in contest.teams.values() {
            q.queue.push(team.score())
        }
        q
    }

    pub fn pop_run<'a>(&mut self, contest: &'a mut ContestFile) -> Option<&'a Team> {
        let entry = self.queue.pop();
        match entry {
            None => None,
            Some(score) => match contest.teams.get_mut(&score.team_login) {
                None => panic!("invalid team!"),
                Some(team) => {
                    team.reveal_run_frozen();
                    if team.wait() {
                        self.queue.push(team.score());
                    }
                    return Some(team);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;

    quickcheck! {
        fn problem_with_runs_is_the_same_as_revealed(answers : Vec<Answer>) -> bool {
            let mut p1 = Problem::empty();
            let mut p2 = Problem::empty();
            println!("------------------------------");
            println!("answers={:?}", answers);
            for a in &answers {
                p1.add_run_problem(a.clone());
                p2.add_run_frozen(a.clone());
            }
            println!("p1={:?}", p1);
            while p2.wait() {
                p2.reveal_run_frozen();

            }
            println!("p2={:?}", p2);

            // p2.answers.clear();

            println!("p2={:?}", p2);
            println!("p1==p2= {}", p1==p2);

            p1 == p2
        }
    }

    #[test]
    fn tree_test() {
        let mut t = BTreeMap::new();
        t.entry(1).or_insert(2);

        assert_eq!(t[&1], 2);

        t.entry(1).or_insert(3);

        assert_eq!(t[&1], 2);
    }
}
