use futures::Stream;
use pin_project::pin_project;
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    thread::{self, JoinHandle},
    time::Duration,
};
use stream_proc_macros::stream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::{wrappers::ReceiverStream, StreamExt, Timeout};

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{event::Event, stream::push_timeout_stream::PushTimeoutStream};

/// Creates an [InputEvent] with a tokio channel.
pub fn input_channel<T>() -> (Sender<T>, InputEvent<T>) {
    let (tx, rx) = channel::<T>(1);
    (tx, InputEvent { receiver: rx })
}

/// Creates an [InputEvent] from an input tokio [Receiver].
pub fn input<T>(receiver: Receiver<T>) -> InputEvent<T> {
    InputEvent { receiver }
}

/// # Input event.
///
/// An input event from a tokio receiver.
/// It implements Event trait, allowing to sample it synchronously
/// (creating a pull stream [InputPull]) or asynchronously
/// (creating a push stream [InputPush]).
pub struct InputEvent<T> {
    receiver: Receiver<T>,
}
impl<T> Event<T> for InputEvent<T>
where
    T: Send + 'static,
{
    type PullStream = InputPull<T>;
    type PushStream = InputPush<T>;
    type PushTimeoutStream = InputPushTimeout<T>;

    fn pull(mut self) -> Self::PullStream {
        let value = Arc::new(Mutex::new(None));
        let _handler = thread::spawn({
            let value = value.clone();
            move || loop {
                let new = self.receiver.blocking_recv().expect("should not end");
                value
                    .lock()
                    .expect("another user of this mutex panicked while holding the mutex")
                    .replace(new);
            }
        });
        InputPull { value, _handler }
    }

    fn push(self) -> Self::PushStream {
        let stream = ReceiverStream::new(self.receiver);
        InputPush { stream }
    }

    fn push_timeout(self, dur: Duration) -> Self::PushTimeoutStream {
        let stream = ReceiverStream::new(self.receiver).timeout(dur);
        InputPushTimeout { stream }
    }
}

/// # Input event's pull stream.
///
/// Created by the method [InputEvent::pull].
pub struct InputPull<T> {
    value: Arc<Mutex<Option<T>>>,
    _handler: JoinHandle<()>,
}
impl<T> PullStream for InputPull<T> {
    type Item = Option<T>;

    fn pick(&mut self) -> Self::Item {
        self.value
            .lock()
            .expect("another user of this mutex panicked while holding the mutex")
            .take()
    }
}

/// # Input event's push stream.
///
/// Created by the method [InputEvent::push].
#[pin_project(project = InputPushProj)]
#[stream(push, item = T)]
pub struct InputPush<T> {
    #[pin]
    stream: ReceiverStream<T>,
}
impl<T> PushStream for InputPush<T> {
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_next(cx) {
            Poll::Ready(Some(value)) => Poll::Ready(value),
            Poll::Ready(None) => panic!("should not end"),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// # Input event's push timeout stream.
///
/// Created by the method [InputEvent::push_timeout].
#[pin_project(project = InputPushTimeoutProj)]
#[stream(timeout, item = Option<T>)]
pub struct InputPushTimeout<T> {
    #[pin]
    stream: Timeout<ReceiverStream<T>>,
}
impl<T> PushTimeoutStream for InputPushTimeout<T> {
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_next(cx) {
            Poll::Ready(Some(value)) => match value {
                Ok(value) => Poll::Ready(Some(value)),
                Err(_) => Poll::Ready(None),
            },
            Poll::Ready(None) => panic!("should not end"),
            Poll::Pending => Poll::Pending,
        }
    }
}
