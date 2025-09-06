use itertools::Itertools;
use std::collections::HashSet;

use crate::{Letter, RunTuple};

pub fn annotate_first_solved<'t>(
    solved: &mut HashSet<Letter>,
    runs: impl Iterator<Item = &'t mut RunTuple>,
) {
    for run in runs.sorted_by_key(|r| r.order) {
        if let crate::Answer::Yes { is_first, .. } = &mut run.answer
            && !solved.contains(&run.prob)
        {
            solved.insert(run.prob.clone());
            *is_first = true;
        }
    }
}
