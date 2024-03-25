use crate::*;

use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
struct Revelation {
    contest: ContestFile,
    runs: RunsFile,
    runs_queue: RunsQueue,
}

#[derive(Debug)]
pub struct RevelationDriver {
    starting_point: Revelation,
    revelation: Revelation,
    step: u32,
}

impl RevelationDriver {
    pub fn new(contest: ContestFile, runs: RunsFile) -> Result<Self, ContestError> {
        let mut revelation = Revelation::new(contest, runs);
        revelation.apply_all_runs_before_frozen()?;

        Ok(Self {
            starting_point: revelation.clone(),
            revelation,
            step: 0,
        })
    }

    pub fn reveal_step(&mut self) -> Result<(), ContestError> {
        self.revelation.apply_one_run_from_queue();
        self.step += 1;
        self.revelation.contest.recalculate_placement_no_filter()
    }

    pub fn peek(&self) -> Option<&String> {
        self.revelation.runs_queue.peek()
    }

    pub fn reveal_top_n(&mut self, n: usize) -> Result<(), ContestError> {
        let steps = self.revelation.apply_runs_from_queue_n(n)?;
        self.step += steps;
        Ok(())
    }

    pub fn jump_team_forward(&mut self) -> Result<(), ContestError> {
        if let Some(center) = self.peek().cloned() {
            while self.peek().is_some_and(|c| c == &center) {
                self.revelation.apply_one_run_from_queue();
                self.step += 1;
            }
            self.revelation.contest.recalculate_placement_no_filter()?;
        }
        Ok(())
    }

    pub fn contest(&self) -> &ContestFile {
        &self.revelation.contest
    }

    pub fn len(&self) -> usize {
        self.revelation.runs_queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.revelation.runs_queue.is_empty()
    }

    pub fn restart(&mut self) {
        self.revelation = self.starting_point.clone();
        self.step = 0;
    }

    pub fn back_one(&mut self) -> Result<(), ContestError> {
        if self.step > 0 {
            self.revelation = self.starting_point.clone();
            self.step -= 1;
            for _ in 0..self.step {
                self.revelation.apply_one_run_from_queue();
            }
            self.revelation.contest.recalculate_placement_no_filter()?;
        }
        Ok(())
    }
}

impl Revelation {
    fn new(contest: ContestFile, runs: RunsFile) -> Self {
        Self {
            contest,
            runs,
            runs_queue: RunsQueue::empty(),
        }
    }

    fn apply_all_runs_before_frozen(&mut self) -> Result<(), ContestError> {
        for run in &self.runs.sorted() {
            if run.time < self.contest.score_freeze_time {
                self.contest.apply_run(run);
            } else {
                self.contest.apply_run_frozen(run);
            }
        }
        self.runs_queue = RunsQueue::setup_queue(&self.contest);
        self.contest.recalculate_placement_no_filter()
    }

    fn apply_one_run_from_queue(&mut self) {
        self.runs_queue.pop_run(&mut self.contest);
    }

    fn apply_runs_from_queue_n(&mut self, n: usize) -> Result<u32, ContestError> {
        let mut count = 0;
        while self.runs_queue.queue.len() > n {
            count += 1;
            self.apply_one_run_from_queue();
        }
        self.contest.recalculate_placement_no_filter()?;
        Ok(count)
    }
}

#[derive(Debug, Clone)]
struct RunsQueue {
    queue: BinaryHeap<Score>,
}

impl RunsQueue {
    fn empty() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    fn len(&self) -> usize {
        self.queue.len()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn peek(&self) -> Option<&String> {
        self.queue.peek().map(|s| &s.team_login)
    }

    fn setup_queue(contest: &ContestFile) -> Self {
        let mut q = Self::empty();
        for team in contest.teams.values() {
            q.queue.push(team.score())
        }
        q
    }

    fn pop_run(&mut self, contest: &mut ContestFile) {
        let entry = self.queue.pop();
        match entry {
            None => (),
            Some(score) => match contest.teams.get_mut(&score.team_login) {
                None => panic!("invalid team!"),
                Some(team) => {
                    if team.reveal_run_frozen() {
                        self.queue.push(team.score());
                    }
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
