use futures::{Sink, Stream};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll, ready},
    time::Duration,
};
use tokio::time::{Instant, Sleep, sleep};

/// Handles a stream timeout, which always ends the stream and triggers a reconnection.
pub trait StreamTimeoutHandler {
    /// Handles the timeout of the stream it was given to.
    fn handle(self);
}

impl<F> StreamTimeoutHandler for F
where
    F: FnOnce(),
{
    #[inline]
    fn handle(self) {
        self()
    }
}

/// Stream wrapper that applies a "consecutive item timeout" to the inner stream.
///
/// The timer restarts on every item. If no item arrives before it elapses, the
/// [`StreamTimeoutHandler`] runs once and the stream ends (triggers reconnection).
#[derive(Debug)]
#[pin_project]
pub struct OnStreamTimeout<S, TimeoutHandler> {
    #[pin]
    socket: S,
    #[pin]
    sleep: Sleep,
    timeout_next_item: Duration,
    on_timeout: Option<TimeoutHandler>,
}

impl<S, TimeoutHandler> OnStreamTimeout<S, TimeoutHandler> {
    pub fn new(socket: S, timeout_next_item: Duration, on_timeout: TimeoutHandler) -> Self {
        Self {
            socket,
            sleep: sleep(timeout_next_item),
            timeout_next_item,
            on_timeout: Some(on_timeout),
        }
    }
}

impl<S, TimeoutHandler> Stream for OnStreamTimeout<S, TimeoutHandler>
where
    S: Stream,
    TimeoutHandler: StreamTimeoutHandler,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if this.on_timeout.is_none() {
            return Poll::Ready(None);
        }

        match this.socket.poll_next(cx) {
            Poll::Ready(Some(item)) => {
                if let Some(deadline) = Instant::now().checked_add(*this.timeout_next_item) {
                    this.sleep.reset(deadline);
                }
                return Poll::Ready(Some(item));
            }
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => {}
        }

        ready!(this.sleep.poll(cx));

        if let Some(on_timeout) = this.on_timeout.take() {
            on_timeout.handle();
        }
        Poll::Ready(None)
    }
}

impl<St, TimeoutHandler, Item> Sink<Item> for OnStreamTimeout<St, TimeoutHandler>
where
    St: Sink<Item>,
{
    type Error = St::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().socket.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        self.project().socket.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().socket.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().socket.poll_close(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ReconnectingSocket, on_stream_err::StreamErrorAction, update::SocketUpdate};
    use futures::{SinkExt, StreamExt, stream};
    use std::{cell::Cell, convert::Infallible, pin::pin, rc::Rc};
    use tokio::{sync::mpsc, time::advance};
    use tokio_stream::wrappers::UnboundedReceiverStream;
    use tokio_test::{assert_pending, assert_ready, assert_ready_eq};

    type TestError = &'static str;

    const TIMEOUT: Duration = Duration::from_secs(10);

    #[derive(Debug)]
    struct TestSocket {
        items: UnboundedReceiverStream<Result<i32, TestError>>,
        sent_tx: mpsc::UnboundedSender<i32>,
    }

    impl Stream for TestSocket {
        type Item = Result<i32, TestError>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.items.poll_next_unpin(cx)
        }
    }

    impl Sink<i32> for TestSocket {
        type Error = Infallible;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, item: i32) -> Result<(), Self::Error> {
            self.sent_tx.send(item).unwrap();
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    fn test_socket() -> (
        TestSocket,
        mpsc::UnboundedSender<Result<i32, TestError>>,
        mpsc::UnboundedReceiver<i32>,
    ) {
        let (items_tx, items_rx) = mpsc::unbounded_channel();
        let (sent_tx, sent_rx) = mpsc::unbounded_channel();

        let socket = TestSocket {
            items: UnboundedReceiverStream::new(items_rx),
            sent_tx,
        };

        (socket, items_tx, sent_rx)
    }

    fn counting_handler(timeouts: &Rc<Cell<u32>>) -> impl FnOnce() + Clone + 'static {
        let timeouts = Rc::clone(timeouts);
        move || timeouts.set(timeouts.get() + 1)
    }

    fn reconnect_on_fatal(error: &TestError) -> StreamErrorAction {
        if *error == "fatal" {
            StreamErrorAction::Reconnect
        } else {
            StreamErrorAction::Continue
        }
    }

    #[tokio::test(start_paused = true)]
    async fn test_on_stream_timeout_resets_timer_on_each_item() {
        let waker = futures::task::noop_waker_ref();
        let mut cx = Context::from_waker(waker);
        let timeouts = Rc::new(Cell::new(0));
        let (tx, rx) = mpsc::unbounded_channel::<i32>();
        let rx = UnboundedReceiverStream::new(rx);
        let mut stream = pin!(OnStreamTimeout::new(
            rx,
            TIMEOUT,
            counting_handler(&timeouts)
        ));

        assert_pending!(stream.poll_next_unpin(&mut cx));

        advance(Duration::from_secs(9)).await;
        tx.send(1).unwrap();
        assert_ready_eq!(stream.poll_next_unpin(&mut cx), Some(1));

        advance(Duration::from_secs(9)).await;
        assert_pending!(stream.poll_next_unpin(&mut cx));

        advance(Duration::from_secs(9)).await;
        tx.send(2).unwrap();
        assert_ready_eq!(stream.poll_next_unpin(&mut cx), Some(2));

        advance(Duration::from_secs(9)).await;
        assert_pending!(stream.poll_next_unpin(&mut cx));
        assert_eq!(timeouts.get(), 0);
    }

    #[tokio::test(start_paused = true)]
    async fn test_on_stream_timeout_runs_handler_once_and_ends() {
        let waker = futures::task::noop_waker_ref();
        let mut cx = Context::from_waker(waker);
        let timeouts = Rc::new(Cell::new(0));
        let (tx, rx) = mpsc::unbounded_channel::<i32>();
        let rx = UnboundedReceiverStream::new(rx);
        let mut stream = pin!(OnStreamTimeout::new(
            rx,
            TIMEOUT,
            counting_handler(&timeouts)
        ));

        tx.send(1).unwrap();
        assert_ready_eq!(stream.poll_next_unpin(&mut cx), Some(1));

        advance(TIMEOUT).await;
        assert_ready_eq!(stream.poll_next_unpin(&mut cx), None);
        assert_eq!(timeouts.get(), 1);

        tx.send(2).unwrap();
        advance(TIMEOUT).await;
        assert_ready_eq!(stream.poll_next_unpin(&mut cx), None);
        assert_ready_eq!(stream.poll_next_unpin(&mut cx), None);
        assert_eq!(timeouts.get(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn test_on_stream_timeout_inner_end_skips_handler() {
        let waker = futures::task::noop_waker_ref();
        let mut cx = Context::from_waker(waker);
        let timeouts = Rc::new(Cell::new(0));
        let (tx, rx) = mpsc::unbounded_channel::<i32>();
        let rx = UnboundedReceiverStream::new(rx);
        let mut stream = pin!(OnStreamTimeout::new(
            rx,
            TIMEOUT,
            counting_handler(&timeouts)
        ));

        drop(tx);
        assert_ready_eq!(stream.poll_next_unpin(&mut cx), None);

        advance(TIMEOUT).await;
        assert_ready_eq!(stream.poll_next_unpin(&mut cx), None);
        assert_eq!(timeouts.get(), 0);
    }

    #[tokio::test(start_paused = true)]
    async fn test_on_stream_timeout_max_duration_never_elapses() {
        let waker = futures::task::noop_waker_ref();
        let mut cx = Context::from_waker(waker);
        let timeouts = Rc::new(Cell::new(0));
        let (tx, rx) = mpsc::unbounded_channel::<i32>();
        let rx = UnboundedReceiverStream::new(rx);
        let mut stream = pin!(OnStreamTimeout::new(
            rx,
            Duration::MAX,
            counting_handler(&timeouts)
        ));

        tx.send(1).unwrap();
        assert_ready_eq!(stream.poll_next_unpin(&mut cx), Some(1));

        advance(Duration::from_secs(60 * 60 * 24 * 365)).await;
        assert_pending!(stream.poll_next_unpin(&mut cx));
        assert_eq!(timeouts.get(), 0);
    }

    #[tokio::test(start_paused = true)]
    async fn test_on_stream_timeout_passes_sink_through() {
        let (socket, _items_tx, mut sent_rx) = test_socket();
        let mut stream = pin!(OnStreamTimeout::new(socket, TIMEOUT, || {}));

        stream.send(7).await.unwrap();

        assert_eq!(sent_rx.try_recv(), Ok(7));
    }

    #[tokio::test(start_paused = true)]
    async fn test_on_stream_timeout_composes_with_on_stream_err_and_socket_updates() {
        let waker = futures::task::noop_waker_ref();
        let mut cx = Context::from_waker(waker);
        let timeouts = Rc::new(Cell::new(0));
        let (socket_a, items_tx_a, mut sent_rx_a) = test_socket();
        let (socket_b, items_tx_b, mut sent_rx_b) = test_socket();
        let (socket_c, items_tx_c, _sent_rx_c) = test_socket();
        let updates = stream::iter([socket_a, socket_b, socket_c])
            .on_stream_timeout(TIMEOUT, counting_handler(&timeouts))
            .on_stream_err(reconnect_on_fatal)
            .with_socket_updates::<_, i32>();
        let mut updates = pin!(updates);

        let Some(SocketUpdate::Connected(mut sink_a)) =
            assert_ready!(updates.poll_next_unpin(&mut cx))
        else {
            panic!("expected SocketUpdate::Connected");
        };
        sink_a.send(1).await.unwrap();
        items_tx_a.send(Ok(10)).unwrap();
        items_tx_a.send(Err("non-fatal")).unwrap();
        assert_eq!(sent_rx_a.try_recv(), Ok(1));
        assert!(matches!(
            assert_ready!(updates.poll_next_unpin(&mut cx)),
            Some(SocketUpdate::Item(Ok(10)))
        ));
        assert!(matches!(
            assert_ready!(updates.poll_next_unpin(&mut cx)),
            Some(SocketUpdate::Item(Err("non-fatal")))
        ));

        assert!(updates.poll_next_unpin(&mut cx).is_pending());
        advance(TIMEOUT).await;
        assert!(matches!(
            assert_ready!(updates.poll_next_unpin(&mut cx)),
            Some(SocketUpdate::Reconnecting)
        ));
        assert_eq!(timeouts.get(), 1);

        let Some(SocketUpdate::Connected(mut sink_b)) =
            assert_ready!(updates.poll_next_unpin(&mut cx))
        else {
            panic!("expected SocketUpdate::Connected");
        };
        sink_b.send(2).await.unwrap();
        items_tx_b.send(Err("fatal")).unwrap();
        assert_eq!(sent_rx_b.try_recv(), Ok(2));
        assert!(matches!(
            assert_ready!(updates.poll_next_unpin(&mut cx)),
            Some(SocketUpdate::Reconnecting)
        ));
        assert_eq!(timeouts.get(), 1);

        assert!(matches!(
            assert_ready!(updates.poll_next_unpin(&mut cx)),
            Some(SocketUpdate::Connected(_))
        ));
        items_tx_c.send(Ok(30)).unwrap();
        assert!(matches!(
            assert_ready!(updates.poll_next_unpin(&mut cx)),
            Some(SocketUpdate::Item(Ok(30)))
        ));
        advance(TIMEOUT).await;
        assert!(matches!(
            assert_ready!(updates.poll_next_unpin(&mut cx)),
            Some(SocketUpdate::Reconnecting)
        ));
        assert_eq!(timeouts.get(), 2);

        assert!(assert_ready!(updates.poll_next_unpin(&mut cx)).is_none());
    }
}
