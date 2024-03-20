use data::{ContestFile, RunsFile, RunsPanelItem};
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use leptos::*;

use crate::api::{create_contest, create_runs};

pub fn provide_contest() -> (
    ReadSignal<Option<ContestFile>>,
    ReadSignal<Vec<RunsPanelItem>>,
) {
    let (contest_signal, set_contest_signal) = create_signal::<Option<ContestFile>>(None);
    let (runs_panel_signal, set_runs_panel_signal) = create_signal::<Vec<RunsPanelItem>>(vec![]);

    spawn_local(async move {
        let original_contest_file = create_contest().await;
        let mut runs_file = RunsFile::empty();

        let mut runs_stream = create_runs().ready_chunks(100_000);

        loop {
            TimeoutFuture::new(1_000).await;
            // get a new batch of runs
            let next_batch = runs_stream.next().await;
            let size = next_batch.as_ref().map(|v| v.len()).unwrap_or_default();
            leptos_dom::logging::console_log(&format!("read next {size:?} runs"));

            if let Some(next_batch) = next_batch {
                let mut fresh_runs = vec![];
                for run_tuple in next_batch {
                    if runs_file.refresh_1(&run_tuple) {
                        fresh_runs.push(run_tuple);
                    }
                }

                if !fresh_runs.is_empty() {
                    let mut fresh_contest = original_contest_file.clone();

                    let mut runs = runs_file.sorted();
                    for r in &runs {
                        fresh_contest.apply_run(r);
                    }

                    fresh_contest
                        .recalculate_placement(None)
                        .expect("Should recalculate placement");

                    runs.reverse();

                    set_runs_panel_signal.set(
                        runs.into_iter()
                            .filter_map(|r| fresh_contest.build_panel_item(&r).ok())
                            .collect(),
                    );
                    set_contest_signal.set(Some(fresh_contest));
                }
            }
        }
    });

    (contest_signal, runs_panel_signal)
}

pub fn provide_runs() -> ReadSignal<RunsFile> {
    let runs_message = create_runs();

    let mut messages_stream = runs_message.ready_chunks(100_000);

    let (runs_file, set_runs_file) = create_signal::<RunsFile>(RunsFile::empty());

    spawn_local(async move {
        loop {
            TimeoutFuture::new(1_000).await;
            let next_chunk = messages_stream.next().await;
            let size = next_chunk.as_ref().map(|v| v.len()).unwrap_or_default();
            leptos_dom::logging::console_log(&format!("read next {size:?} runs"));
            if let Some(next_chunk) = next_chunk {
                set_runs_file.update(|rf| {
                    for run_tuple in next_chunk {
                        rf.refresh_1(&run_tuple);
                    }
                });
            }
        }
    });

    runs_file
}