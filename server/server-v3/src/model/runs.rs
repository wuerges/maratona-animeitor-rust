use std::{collections::HashSet, sync::Arc, time::Duration};

use futures::Stream;
use itertools::Itertools;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

pub struct Runs {
    known: Arc<RwLock<HashSet<sdk::Run>>>,
    sender: membroadcast::Sender<sdk::Run>,
}

impl Runs {
    pub async fn stream(&self) -> impl Stream<Item = Vec<sdk::Run>> {
        self.sender
            .subscribe()
            .await
            .recv_stream()
            .chunks_timeout(1_000_000, Duration::from_secs(1))
    }

    pub async fn new() -> Self {
        let (sender, _) = membroadcast::channel(1_000_000).await;

        Self {
            known: Arc::new(RwLock::new(HashSet::new())),
            sender,
        }
    }

    pub async fn push_ordered(&self, new_runs: Vec<sdk::Run>) {
        let fresh = {
            let read = self.known.read().await;
            new_runs
                .into_iter()
                .filter(|r| read.contains(r))
                .collect_vec()
        };

        let mut write = self.known.write().await;

        for run in fresh {
            if write.insert(run.clone()) {
                self.sender.send_memo(run).await;
            }
        }
    }

    pub async fn reset(&self) {
        self.known.write().await.clear();
        self.sender.clear().await;
    }
}
