use itertools::Itertools;
use std::collections::{BinaryHeap, HashSet, VecDeque};

use crate::annotate_first_solved::annotate_first_solved;
use crate::scoring::{ContestError, ContestFileExt, Score, TeamExt};
use data::{ContestFile, RunTuple, RunsFile};

#[derive(Debug, Clone)]
struct Revelation {
    contest: ContestFile,
    runs_queue: RunsQueue,
}

#[derive(Debug)]
pub struct RevelationDriver {
    starting_point: Revelation,
    revelation: Revelation,
    step: u32,
    checkpoints: VecDeque<(u32, Revelation)>,
}

impl RevelationDriver {
    pub fn new(contest: ContestFile, mut runs: RunsFile) -> Self {
        runs.filter_teams(&contest);
        let revelation = Revelation::new(contest, runs.sorted());

        Self {
            starting_point: revelation.clone(),
            revelation,
            step: 0,
            checkpoints: VecDeque::new(),
        }
    }

    pub fn reveal_step(&mut self) {
        self.advance();
        self.revelation.contest.recalculate_placement()
    }

    pub fn peek(&self) -> Option<&String> {
        self.revelation.runs_queue.peek()
    }

    pub fn reveal_top_n(&mut self, n: usize) -> Result<(), ContestError> {
        while self.len() > n {
            self.advance();
        }
        self.revelation.contest.recalculate_placement();
        Ok(())
    }

    pub fn jump_team_forward(&mut self) {
        if let Some(center) = self.peek().cloned() {
            while self.peek().is_some_and(|c| c == &center) {
                self.advance();
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
        self.checkpoints.clear();
    }

    // Checkpoints contain mutable scoring state and the queue, never the raw run history.
    fn advance(&mut self) {
        self.revelation.apply_one_run_from_queue();
        self.step += 1;
        if self.step % 100 == 0 {
            self.checkpoints.retain(|(step, _)| *step < self.step);
            self.checkpoints
                .push_back((self.step, self.revelation.clone()));
            if self.checkpoints.len() > 8 {
                self.checkpoints.pop_front();
            }
        }
    }

    pub fn back_one(&mut self) {
        if self.step == 0 {
            return;
        }
        let target = self.step - 1;
        let (step, state) = self
            .checkpoints
            .iter()
            .rev()
            .find(|(step, _)| *step <= target)
            .map(|(step, state)| (*step, state.clone()))
            .unwrap_or_else(|| (0, self.starting_point.clone()));
        self.revelation = state;
        self.step = step;
        self.checkpoints.retain(|(step, _)| *step <= target);
        while self.step < target {
            self.advance();
        }
        self.revelation.contest.recalculate_placement();
    }
}

impl Revelation {
    fn new(contest: ContestFile, mut runs: Vec<RunTuple>) -> Self {
        let mut solved = HashSet::new();
        annotate_first_solved(&mut solved, runs.iter_mut().sorted_by_key(|r| r.order));

        let mut state = Self {
            contest,
            runs_queue: RunsQueue::empty(),
        };
        state.apply_all_runs_before_frozen(&runs);
        state
    }

    fn apply_all_runs_before_frozen(&mut self, runs: &[RunTuple]) {
        for run in runs {
            if run.time < self.contest.score_freeze_time {
                self.contest.apply_run(run);
            } else {
                self.contest.apply_run_frozen(run);
            }
        }
        self.runs_queue = RunsQueue::setup_queue(&self.contest);
        self.contest.recalculate_placement()
    }

    fn apply_one_run_from_queue(&mut self) {
        self.runs_queue.pop_run(&mut self.contest);
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
        let wrong_penalty = contest.penalty_per_wrong_answer;
        let entry = self.queue.pop();
        match entry {
            None => (),
            Some(score) => match contest.teams.get_mut(&score.team_login) {
                None => panic!("invalid team!"),
                Some(team) => {
                    if team.reveal_run_frozen(wrong_penalty) {
                        self.queue.push(team.score());
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod checkpoint_tests {
    use super::*;
    use data::{Answer, Team};

    fn fixture() -> (ContestFile, RunsFile) {
        let contest = ContestFile {
            contest_name: "checkpoint test".into(),
            teams: (0..150)
                .map(|i| {
                    let login = format!("t{i:04}");
                    (
                        login.clone(),
                        Team {
                            login,
                            escola: "School".into(),
                            name: format!("Team {i}"),
                            placement: i + 1,
                            placement_global: i + 1,
                            problems: Default::default(),
                            id: 0,
                        },
                    )
                })
                .collect(),
            current_time: 300,
            maximum_time: 300,
            score_freeze_time: 100,
            penalty_per_wrong_answer: 1200,
            number_problems: 2,
        };
        let runs = RunsFile::from_runs(
            (0..1500)
                .map(|i| RunTuple {
                    id: i,
                    order: i as u64,
                    time: 101 + i / 15,
                    team_login: format!("t{:04}", i % 150),
                    prob: if i % 2 == 0 { "A" } else { "B" }.parse().unwrap(),
                    answer: if i >= 1350 {
                        Answer::Yes {
                            time: 200,
                            is_first: false,
                            run_id: i,
                        }
                    } else {
                        Answer::No { run_id: i }
                    },
                })
                .collect(),
        );
        (contest, runs)
    }

    fn assert_same(actual: &RevelationDriver, expected: &RevelationDriver) {
        assert_eq!(actual.step, expected.step);
        assert_eq!(actual.peek(), expected.peek());
        assert_eq!(actual.len(), expected.len());
        for (login, team) in &actual.contest().teams {
            let reference = &expected.contest().teams[login];
            assert_eq!(team.score(), reference.score());
            assert_eq!(team.placement_global, reference.placement_global);
            for (letter, problem) in &team.problems {
                let other = &reference.problems[letter];
                assert_eq!(problem.solved_first, other.solved_first);
                assert_eq!(problem.answers, other.answers);
                assert_eq!(problem.submissions, other.submissions);
            }
        }
    }

    #[test]
    fn checkpoints_match_full_replay_across_eviction_and_backtracking() {
        let (contest, runs) = fixture();
        let mut actual = RevelationDriver::new(contest.clone(), runs.clone());
        actual.reveal_top_n(0).unwrap();
        assert_eq!(actual.checkpoints.len(), 8);
        for _ in 0..220 {
            actual.back_one();
            let mut reference = RevelationDriver::new(contest.clone(), runs.clone());
            for _ in 0..actual.step {
                reference.reveal_step();
            }
            assert_same(&actual, &reference);
        }
        actual.jump_team_forward();
        let mut reference = RevelationDriver::new(contest.clone(), runs.clone());
        for _ in 0..actual.step {
            reference.reveal_step();
        }
        assert_same(&actual, &reference);
        actual.restart();
        assert!(actual.checkpoints.is_empty());
        assert_same(&actual, &RevelationDriver::new(contest, runs));
    }
}
