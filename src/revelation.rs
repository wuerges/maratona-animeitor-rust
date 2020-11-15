use crate::data::*;

pub struct Revelation {
    pub contest: ContestFile,
    runs: RunsFile,
    runs_queue: RunsQueue,
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
        self.runs_queue.setup_teams(&self.contest);
        self.contest.recalculate_placement().unwrap();
    }

    pub fn apply_all_runs_on_frozen(&mut self) {
        for run in self.runs.sorted() {
            self.contest.apply_run_frozen(run).unwrap();
        }
        self.runs_queue.setup_teams(&self.contest);
        self.contest.recalculate_placement().unwrap();
    }

    pub fn apply_one_run_from_queue(&mut self) {
        let _ = self.runs_queue.pop_run(&mut self.contest);
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
}
