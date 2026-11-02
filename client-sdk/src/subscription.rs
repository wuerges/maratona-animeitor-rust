//! Dropping a subscription cancels its producer even when the stream is idle.
use futures::{
    Stream,
    channel::{mpsc, oneshot},
};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
pub struct Subscription<T> {
    receiver: mpsc::UnboundedReceiver<T>,
    _cancel: oneshot::Sender<()>,
}
impl<T> Stream for Subscription<T> {
    type Item = T;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        Pin::new(&mut self.receiver).poll_next(cx)
    }
}
pub(crate) fn channel<T>() -> (
    mpsc::UnboundedSender<T>,
    Subscription<T>,
    oneshot::Receiver<()>,
) {
    let (sender, receiver) = mpsc::unbounded();
    let (cancel, canceled) = oneshot::channel();
    (
        sender,
        Subscription {
            receiver,
            _cancel: cancel,
        },
        canceled,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dropping_idle_subscription_signals_cancellation() {
        let (_sender, receiver, canceled) = channel::<u8>();
        drop(receiver);
        assert!(futures::executor::block_on(canceled).is_err());
    }
}
