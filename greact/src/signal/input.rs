use futures::Stream;
use pin_project::pin_project;
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
};
use stream_proc_macros::stream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::{wrappers::ReceiverStream, StreamExt, Timeout};

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{signal::Signal, stream::push_timeout_stream::PushTimeoutStream};

/// Creates an [InputSignal] with a mutex inside.
pub fn input_mutex<T>(initial_value: T) -> ((Arc<Mutex<T>>, Sender<T>), InputSignal<T>)
where
    T: Clone,
{
    let (tx, rx) = channel::<T>(1);
    let mutex = Arc::new(Mutex::new(initial_value.clone()));
    (
        (mutex.clone(), tx),
        InputSignal {
            initial_value,
            mutex,
            receiver: rx,
        },
    )
}

/// # Input signal.
///
/// An input signal from a tokio receiver.
/// It implements Signal trait, allowing to sample it synchronously
/// (creating a pull stream [InputPull]) or asynchronously
/// (creating a push stream [InputPush]).
pub struct InputSignal<T>
where
    T: Clone,
{
    initial_value: T,
    mutex: Arc<Mutex<T>>,
    receiver: Receiver<T>,
}
impl<T> Signal<T> for InputSignal<T>
where
    T: Clone + Send + 'static,
{
    type PullStream = InputPull<T>;
    type PushStream = InputPush<T>;
    type PushTimeoutStream = InputPushTimeout<T>;

    fn pull(self) -> Self::PullStream {
        let value = self.mutex.clone();
        InputPull { value }
    }

    fn push(self) -> Self::PushStream {
        let initial_value = Some(self.initial_value);
        let stream = ReceiverStream::new(self.receiver);
        InputPush {
            initial_value,
            stream,
        }
    }

    fn push_timeout(self, dur: Duration) -> Self::PushTimeoutStream {
        let last_value = self.initial_value;
        let stream = ReceiverStream::new(self.receiver).timeout(dur);
        InputPushTimeout {
            first: true,
            last_value,
            stream,
        }
    }
}

/// # Input signal's pull stream.
///
/// Created by the method [InputSignal::pull].
pub struct InputPull<T>
where
    T: Clone,
{
    value: Arc<Mutex<T>>,
}
impl<T> PullStream for InputPull<T>
where
    T: Clone,
{
    type Item = T;

    fn pick(&mut self) -> Self::Item {
        self.value
            .lock()
            .expect("another user of this mutex panicked while holding the mutex")
            .clone()
    }
}

/// # Input signal's push stream.
///
/// Created by the method [InputSignal::push].
#[pin_project(project = InputPushProj)]
#[stream(push, item = T)]
pub struct InputPush<T>
where
    T: Clone,
{
    initial_value: Option<T>,
    #[pin]
    stream: ReceiverStream<T>,
}
impl<T> PushStream for InputPush<T>
where
    T: Clone,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();

        // initialize the stream when first value is pending
        if let Some(initial_value) = project.initial_value.take() {
            match project.stream.poll_next(cx) {
                Poll::Ready(Some(value)) => Poll::Ready(value),
                Poll::Ready(None) => panic!("should not end"),
                Poll::Pending => Poll::Ready(initial_value),
            }
        } else {
            match project.stream.poll_next(cx) {
                Poll::Ready(Some(value)) => Poll::Ready(value),
                Poll::Ready(None) => panic!("should not end"),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

/// # Input signal's push timeout stream.
///
/// Created by the method [InputSignal::push_timeout].
#[pin_project(project = InputPushTimeoutProj)]
#[stream(timeout, item = T)]
pub struct InputPushTimeout<T>
where
    T: Clone,
{
    first: bool,
    last_value: T,
    #[pin]
    stream: Timeout<ReceiverStream<T>>,
}
impl<T> PushTimeoutStream for InputPushTimeout<T>
where
    T: Clone,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();

        // initialize the stream when first value is pending
        if *project.first {
            *project.first = false;
            match project.stream.poll_next(cx) {
                Poll::Ready(Some(Ok(value))) => Poll::Ready(value),
                Poll::Ready(None) => panic!("should not end"),
                _ => Poll::Ready(project.last_value.clone()),
            }
        } else {
            // copy last value when received value is timeout error
            match project.stream.poll_next(cx) {
                Poll::Ready(Some(value)) => match value {
                    Ok(value) => Poll::Ready(value),
                    Err(_) => Poll::Ready(project.last_value.clone()),
                },
                Poll::Ready(None) => panic!("should not end"),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}
