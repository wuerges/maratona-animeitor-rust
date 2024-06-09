use crate::*;

use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
struct Revelation {
    contest: ContestFile,
    runs: RunsFileContest,
    runs_queue: RunsQueue,
}

#[derive(Debug)]
pub struct RevelationDriver {
    starting_point: Revelation,
    revelation: Revelation,
    step: u32,
}

impl RevelationDriver {
    pub fn new(contest: ContestFile, runs: RunsFile) -> Self {
        let runs = runs.into_runs_sede(&contest);
        let mut revelation = Revelation::new(contest, runs);
        revelation.apply_all_runs_before_frozen();

        Self {
            starting_point: revelation.clone(),
            revelation,
            step: 0,
        }
    }

    pub fn reveal_step(&mut self) {
        self.revelation.apply_one_run_from_queue();
        self.step += 1;
        self.revelation.contest.recalculate_placement()
    }

    pub fn peek(&self) -> Option<&String> {
        self.revelation.runs_queue.peek()
    }

    pub fn reveal_top_n(&mut self, n: usize) -> Result<(), ContestError> {
        let steps = self.revelation.apply_runs_from_queue_n(n)?;
        self.step += steps;
        Ok(())
    }

    pub fn jump_team_forward(&mut self) {
        if let Some(center) = self.peek().cloned() {
            while self.peek().is_some_and(|c| c == &center) {
                self.revelation.apply_one_run_from_queue();
                self.step += 1;
            }
            self.revelation.contest.recalculate_placement();
        }
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

    pub fn back_one(&mut self) {
        if self.step > 0 {
            self.revelation = self.starting_point.clone();
            self.step -= 1;
            for _ in 0..self.step {
                self.revelation.apply_one_run_from_queue();
            }
            self.revelation.contest.recalculate_placement();
        }
    }
}

impl Revelation {
    fn new(contest: ContestFile, runs: RunsFileContest) -> Self {
        Self {
            contest,
            runs,
            runs_queue: RunsQueue::empty(),
        }
    }

    fn apply_all_runs_before_frozen(&mut self) {
        for run in self.runs.as_ref().sorted() {
            if run.time < self.contest.score_freeze_time {
                self.contest.apply_run(&run);
            } else {
                self.contest.apply_run_frozen(&run);
            }
        }
        self.runs_queue = RunsQueue::setup_queue(&self.contest);
        self.contest.recalculate_placement()
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
        self.contest.recalculate_placement();
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
