use itertools::Itertools;
use std::collections::HashSet;

use crate::RunTuple;

pub fn annotate_first_solved<'t>(
    solved: &mut HashSet<String>,
    runs: impl Iterator<Item = &'t mut RunTuple>,
) {
    for run in runs.sorted_by_key(|r| r.order) {
        match &mut run.answer {
            crate::Answer::Yes { is_first, .. } => {
                if !solved.contains(&run.prob) {
                    solved.insert(run.prob.clone());
                    *is_first = true;
                }
            }
            _ => (),
        }
    }
}
