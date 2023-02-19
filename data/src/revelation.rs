use crate::*;

use std::collections::BinaryHeap;

struct Revelation {
    contest: ContestFile,
    runs: RunsFile,
    runs_queue: RunsQueue,
}

pub struct RevelationDriver {
    revelation: Revelation,
}

impl RevelationDriver {
    pub fn new(contest: ContestFile, runs: RunsFile) -> Self {
        let mut revelation = Revelation::new(contest, runs);
        revelation.apply_all_runs_before_frozen();

        Self { revelation }
    }

    pub fn reveal_step(&mut self) {
        self.revelation.apply_one_run_from_queue();
        self.revelation
            .contest
            .recalculate_placement_no_filter()
            .unwrap();
    }

    pub fn peek(&self) -> Option<&String> {
        self.revelation.runs_queue.peek()
    }

    pub fn reveal_top_n(&mut self, n: usize) {
        self.revelation.apply_runs_from_queue_n(n);
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
}

impl Revelation {
    fn new(contest: ContestFile, runs: RunsFile) -> Self {
        Self {
            contest,
            runs,
            runs_queue: RunsQueue::empty(),
        }
    }

    fn apply_all_runs_before_frozen(&mut self) {
        for run in &self.runs.sorted() {
            if run.time < self.contest.score_freeze_time {
                self.contest.apply_run(run).unwrap();
            } else {
                self.contest.apply_run_frozen(run).unwrap();
            }
        }
        self.runs_queue = RunsQueue::setup_queue(&self.contest);
        self.contest.recalculate_placement_no_filter().unwrap();
    }

    fn apply_one_run_from_queue(&mut self) {
        self.runs_queue.pop_run(&mut self.contest);
    }

    fn apply_runs_from_queue_n(&mut self, n: usize) {
        while self.runs_queue.queue.len() > n {
            self.apply_one_run_from_queue();
        }
        self.contest.recalculate_placement_no_filter().unwrap();
    }
}

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
