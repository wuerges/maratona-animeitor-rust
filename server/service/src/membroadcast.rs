use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Receiver<T: Clone> {
    rx: broadcast::Receiver<T>,
    messages: Arc<Vec<T>>,
    position: usize,
}

impl<T: Clone> Receiver<T> {
    fn new(rx: broadcast::Receiver<T>, messages: Arc<Vec<T>>) -> Self {
        Self {
            rx,
            messages,
            position: 0,
        }
    }

    pub async fn recv(&mut self) -> Result<T, broadcast::error::RecvError> {
        let front = self.messages.get(self.position).cloned();
        if front.is_some() {
            self.position += 1;
        } else if !self.messages.is_empty() {
            self.messages = Arc::new(Vec::new());
            self.position = 0;
        }

        match front {
            None => self.rx.recv().await,
            Some(message) => Ok(message),
        }
    }
}

#[derive(Clone)]
pub struct Sender<T: Clone> {
    tx: broadcast::Sender<T>,
    messages: Arc<RwLock<Arc<Vec<T>>>>,
}

impl<T: Clone> Sender<T> {
    fn new(tx: broadcast::Sender<T>) -> Self {
        Self {
            tx,
            messages: Arc::new(RwLock::new(Arc::new(Vec::new()))),
        }
    }

    pub(crate) fn send_memo(&self, value: T) -> usize {
        Arc::make_mut(&mut self.messages.write()).push(value.clone());
        self.tx.send(value).unwrap_or(0)
    }

    pub(crate) fn replace_memo(&self, messages: Vec<T>) {
        *self.messages.write() = Arc::new(messages);
    }

    pub(crate) fn subscribe(&self) -> Receiver<T> {
        let rx = self.tx.subscribe();
        let messages = self.messages.read().clone();
        metrics::counter!("server_replay_messages_total").increment(messages.len() as u64);
        Receiver::new(rx, messages)
    }

    #[cfg(test)]
    pub fn receiver_count_memo(&self) -> usize {
        self.tx.receiver_count()
    }
}

pub(crate) fn channel<T: Clone>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = broadcast::channel(capacity);
    let mem_tx = Sender::new(tx);
    let mem_rx = Receiver::new(rx, mem_tx.messages.read().clone());
    (mem_tx, mem_rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn replay_is_shared_and_stable_across_updates() {
        let (tx, _) = channel(16);
        tx.replace_memo(vec![1, 2]);
        let mut first = tx.subscribe();
        let second = tx.subscribe();
        assert!(Arc::ptr_eq(&first.messages, &second.messages));
        tx.send_memo(3);
        tx.replace_memo(vec![2, 3]);
        assert_eq!(first.recv().await.unwrap(), 1);
        assert_eq!(first.recv().await.unwrap(), 2);
        assert_eq!(first.recv().await.unwrap(), 3);
        assert!(first.messages.is_empty());
        let mut fresh = tx.subscribe();
        assert_eq!(fresh.recv().await.unwrap(), 2);
        assert_eq!(fresh.recv().await.unwrap(), 3);
    }

    #[tokio::test]
    async fn test_mem_broadcast() {
        let (tx, mut rx1) = channel(1_000_000);

        let t1 = tokio::spawn(async move {
            println!("rx1 started");
            assert_eq!(rx1.recv().await.unwrap(), 10);
            assert_eq!(rx1.recv().await.unwrap(), 20);
            assert_eq!(rx1.recv().await.unwrap(), 30);
            println!("rx1 finished");
        });

        assert_eq!(tx.receiver_count_memo(), 1);

        tx.send_memo(10);
        println!("send_memo 10");

        let mut rx2 = tx.subscribe();
        let t2 = tokio::spawn(async move {
            println!("rx2 started");
            assert_eq!(rx2.recv().await.unwrap(), 10);
            assert_eq!(rx2.recv().await.unwrap(), 20);
            assert_eq!(rx2.recv().await.unwrap(), 30);
            println!("rx2 finished");
        });
        assert_eq!(tx.receiver_count_memo(), 2);

        tx.send_memo(20);
        println!("send_memo 20");

        let mut rx3 = tx.subscribe();
        let t3 = tokio::spawn(async move {
            println!("rx3 started");
            assert_eq!(rx3.recv().await.unwrap(), 10);
            assert_eq!(rx3.recv().await.unwrap(), 20);
            assert_eq!(rx3.recv().await.unwrap(), 30);
            println!("rx3 finished");
        });

        assert_eq!(tx.receiver_count_memo(), 3);
        tx.send_memo(30);
        println!("send_memo 30");

        t1.await.expect("t1");
        t2.await.expect("t2");
        t3.await.expect("t3");
    }
}
