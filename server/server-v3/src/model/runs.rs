use std::{
    collections::HashSet,
    sync::{Arc, atomic::AtomicU64},
    time::Duration,
};

use futures::Stream;
use itertools::Itertools;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

pub struct Runs {
    known: Arc<RwLock<HashSet<sdk::Run>>>,
    count: AtomicU64,
    sender: membroadcast::Sender<sdk::Run>,
    timeout: Duration,
}

impl Runs {
    pub fn stream(&self) -> impl Stream<Item = Vec<sdk::Run>> {
        self.sender
            .subscribe()
            .recv_stream()
            .chunks_timeout(1_000_000, Duration::from_secs(1))
    }

    pub fn new(timeout: Duration) -> Self {
        let (sender, _) = membroadcast::channel(1_000_000);

        Self {
            known: Arc::new(RwLock::new(HashSet::new())),
            sender,
            count: AtomicU64::new(0),
            timeout,
        }
    }

    pub async fn push_ordered(&self, new_runs: Vec<sdk::Run>) {
        let fresh = {
            let read = self.known.read().await;
            new_runs
                .into_iter()
                .filter(|r| read.contains(&r))
                .collect_vec()
        };

        let mut write = self.known.write().await;

        for mut run in fresh {
            // Order stamp is added with order = 0
            run.order = 0;
            if write.insert(run.clone()) {
                run.order = self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                self.sender.send_memo(run);
            }
        }
    }
}
