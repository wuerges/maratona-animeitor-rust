use tokio::sync::broadcast;
use std::collections::VecDeque;

pub struct MemReceiver<T: Clone> {
    rx : broadcast::Receiver<T>,
    messages: VecDeque<T>
}

impl<T: Clone> MemReceiver<T> {
    fn new(rx : broadcast::Receiver<T>, messages: &Vec<T>) -> Self {
        let mut deque = VecDeque::new();
        for m in messages {
            deque.push_back(m.clone())
        }
        Self {
            rx,
            messages: deque
        }
    }

    pub async fn recv(&mut self) -> Result<T, broadcast::RecvError> {
        let front = self.messages.pop_front();

        match front {
            None => self.rx.recv().await,
            Some(message) => Ok(message)
        }
    }
}

pub struct MemSender<T: Clone> {
    tx : broadcast::Sender<T>,
    messages: Vec<T>
}

impl<T: Clone> MemSender<T> {
    fn new(tx : broadcast::Sender<T>) -> Self {
        Self {
            tx,
            messages: Vec::new()
        }
    }

    pub fn send_memo(&mut self, value: T) -> Result<usize, broadcast::SendError<T>> {
        self.messages.push(value.clone());
        self.tx.send(value)
    }

    pub fn subscribe(&self) -> MemReceiver<T> {
        let rx = self.tx.subscribe();
        MemReceiver::new(rx, &self.messages)
    }

    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

pub fn channel<T: Clone>(capacity: usize) -> (MemSender<T>, MemReceiver<T>) {
    let (tx, rx) = broadcast::channel(capacity);
    let mem_tx = MemSender::new(tx);
    let mem_rx = MemReceiver::new(rx, &mem_tx.messages);
    (mem_tx, mem_rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mem_broadcast() {
        let (mut tx, mut rx1) = channel(1_000_000);

        let t1 = tokio::spawn(async move {
            println!("rx1 started");
            assert_eq!(rx1.recv().await.unwrap(), 10);
            assert_eq!(rx1.recv().await.unwrap(), 20);
            assert_eq!(rx1.recv().await.unwrap(), 30);
            println!("rx1 finished");
        });

        assert_eq!(tx.receiver_count(), 1);

        tx.send_memo(10).unwrap();
        println!("send_memo 10");

        let mut rx2 = tx.subscribe();
        let t2 = tokio::spawn(async move {
            println!("rx2 started");
            assert_eq!(rx2.recv().await.unwrap(), 10);
            assert_eq!(rx2.recv().await.unwrap(), 20);
            assert_eq!(rx2.recv().await.unwrap(), 30);
            println!("rx2 finished");
        });
        assert_eq!(tx.receiver_count(), 2);

        tx.send_memo(20).unwrap();
        println!("send_memo 20");

        let mut rx3 = tx.subscribe();
        let t3 = tokio::spawn(async move {
            println!("rx3 started");
            assert_eq!(rx3.recv().await.unwrap(), 10);
            assert_eq!(rx3.recv().await.unwrap(), 20);
            assert_eq!(rx3.recv().await.unwrap(), 30);
            println!("rx3 finished");
        });

        assert_eq!(tx.receiver_count(), 3);
        tx.send_memo(30).unwrap();
        println!("send_memo 30");

        t1.await.expect("t1");
        t2.await.expect("t2");
        t3.await.expect("t3");
    }
}
