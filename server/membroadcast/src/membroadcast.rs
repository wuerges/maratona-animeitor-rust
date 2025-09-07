use futures::Stream;
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::{RwLock, broadcast};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

pub struct Receiver<T: Clone> {
    rx: broadcast::Receiver<T>,
    messages: VecDeque<T>,
}

impl<T: Clone> Receiver<T> {
    fn new(rx: broadcast::Receiver<T>, messages: Vec<T>) -> Self {
        let mut deque = VecDeque::new();
        deque.extend(messages);

        Self {
            rx,
            messages: deque,
        }
    }

    pub async fn recv(&mut self) -> Result<T, broadcast::error::RecvError> {
        let front = self.messages.pop_front();

        match front {
            None => self.rx.recv().await,
            Some(message) => Ok(message),
        }
    }

    pub fn recv_stream(self) -> impl Stream<Item = T>
    where
        T: Send + 'static,
    {
        let broadcast = BroadcastStream::new(self.rx.resubscribe()).filter_map(|m| m.ok());
        tokio_stream::iter(self.messages).chain(broadcast)
    }
}

#[derive(Clone)]
pub struct Sender<T: Clone> {
    tx: broadcast::Sender<T>,
    messages: Arc<RwLock<Vec<T>>>,
}

impl<T: Clone> Sender<T> {
    fn new(tx: broadcast::Sender<T>) -> Self {
        Self {
            tx,
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn send_memo(&self, value: T) -> usize {
        self.messages.write().await.push(value.clone());
        self.tx.send(value).unwrap_or(0)
    }

    pub async fn subscribe(&self) -> Receiver<T> {
        let rx = self.tx.subscribe();
        Receiver::new(rx, self.messages.read().await.clone())
    }

    #[cfg(test)]
    pub fn receiver_count_memo(&self) -> usize {
        self.tx.receiver_count()
    }
}

pub async fn channel<T: Clone>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = broadcast::channel(capacity);
    let mem_tx = Sender::new(tx);
    let mem_rx = Receiver::new(rx, mem_tx.messages.read().await.clone());
    (mem_tx, mem_rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mem_broadcast() {
        let (tx, mut rx1) = channel(1_000_000).await;

        let t1 = tokio::spawn(async move {
            println!("rx1 started");
            assert_eq!(rx1.recv().await.unwrap(), 10);
            assert_eq!(rx1.recv().await.unwrap(), 20);
            assert_eq!(rx1.recv().await.unwrap(), 30);
            println!("rx1 finished");
        });

        assert_eq!(tx.receiver_count_memo(), 1);

        tx.send_memo(10).await;
        println!("send_memo 10");

        let mut rx2 = tx.subscribe().await;
        let t2 = tokio::spawn(async move {
            println!("rx2 started");
            assert_eq!(rx2.recv().await.unwrap(), 10);
            assert_eq!(rx2.recv().await.unwrap(), 20);
            assert_eq!(rx2.recv().await.unwrap(), 30);
            println!("rx2 finished");
        });
        assert_eq!(tx.receiver_count_memo(), 2);

        tx.send_memo(20).await;
        println!("send_memo 20");

        let mut rx3 = tx.subscribe().await;
        let t3 = tokio::spawn(async move {
            println!("rx3 started");
            assert_eq!(rx3.recv().await.unwrap(), 10);
            assert_eq!(rx3.recv().await.unwrap(), 20);
            assert_eq!(rx3.recv().await.unwrap(), 30);
            println!("rx3 finished");
        });

        assert_eq!(tx.receiver_count_memo(), 3);
        tx.send_memo(30).await;
        println!("send_memo 30");

        t1.await.expect("t1");
        t2.await.expect("t2");
        t3.await.expect("t3");
    }
}
