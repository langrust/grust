use pin_project::pin_project;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use stream_proc_macros::stream;

use crate::{
    event::Event,
    signal::Signal,
    stream::{
        pull_stream::PullStream, push_stream::PushStream, push_timeout_stream::PushTimeoutStream,
    },
};

/// Creates a [ZipEventSignal] that zips the given event with the given signal.
pub fn zip_event<U, V, E, S>(event: E, signal: S) -> ZipEventSignal<U, V, E, S>
where
    E: Event<U>,
    S: Signal<V>,
    V: Clone,
{
    ZipEventSignal {
        event,
        signal,
        phantom: PhantomData,
    }
}

/// # ZipEvent signal.
///
/// A signal zipping an event to a signal.
/// It implements Signal trait, allowing to sample it synchronously
/// (creating a pull stream [ZipEventPull]) or asynchronously
/// (creating a push stream [ZipEventPush]).
pub struct ZipEventSignal<U, V, E, S>
where
    E: Event<U>,
    S: Signal<V>,
    V: Clone,
{
    event: E,
    signal: S,
    phantom: PhantomData<(Option<U>, V)>,
}
impl<U, V, E, S> Signal<(Option<U>, V)> for ZipEventSignal<U, V, E, S>
where
    E: Event<U>,
    S: Signal<V>,
    V: Clone,
{
    type PullStream = ZipEventPull<U, V, E::PullStream, S::PullStream>;
    type PushStream = ZipEventPush<U, V, E::PushStream, S::PushStream>;
    type PushTimeoutStream = ZipEventPushTimeout<U, V, E::PushTimeoutStream, S::PushTimeoutStream>;

    fn pull(self) -> Self::PullStream {
        let pull_stream_1 = self.event.pull();
        let pull_stream_2 = self.signal.pull();
        ZipEventPull {
            pull_stream_1,
            pull_stream_2,
        }
    }

    fn push(self) -> Self::PushStream {
        let stream_1 = self.event.push();
        let stream_2 = self.signal.push();
        let prev_value_2 = None;
        ZipEventPush {
            stream_1,
            stream_2,
            prev_value_2,
        }
    }

    fn push_timeout(self, dur: std::time::Duration) -> Self::PushTimeoutStream {
        let stream_1 = self.event.push_timeout(dur);
        let stream_2 = self.signal.push_timeout(dur);
        let prev_value_2 = None;
        ZipEventPushTimeout {
            stream_1,
            stream_2,
            prev_value_2,
        }
    }
}

/// # ZipEvent signal's pull stream.
///
/// Created by the method [ZipEventSignal::pull].
pub struct ZipEventPull<U, V, Pull1, Pull2>
where
    Pull1: PullStream<Item = Option<U>>,
    Pull2: PullStream<Item = V>,
    V: Clone,
{
    pull_stream_1: Pull1,
    pull_stream_2: Pull2,
}
impl<U, V, Pull1, Pull2> PullStream for ZipEventPull<U, V, Pull1, Pull2>
where
    Pull1: PullStream<Item = Option<U>>,
    Pull2: PullStream<Item = V>,
    V: Clone,
{
    type Item = (Option<U>, V);

    fn pick(&mut self) -> Self::Item {
        (self.pull_stream_1.pick(), self.pull_stream_2.pick())
    }
}

/// # ZipEvent signal's push stream.
///
/// Created by the method [ZipEventSignal::push].
#[pin_project(project = ZipEventPushProj)]
#[stream(push, item = (Option<U>, V))]
pub struct ZipEventPush<U, V, Push1, Push2>
where
    Push1: PushStream<Item = U>,
    Push2: PushStream<Item = V>,
    V: Clone,
{
    #[pin]
    stream_1: Push1,
    #[pin]
    stream_2: Push2,
    prev_value_2: Option<V>,
}
impl<U, V, Push1, Push2> PushStream for ZipEventPush<U, V, Push1, Push2>
where
    Push1: PushStream<Item = U>,
    Push2: PushStream<Item = V>,
    V: Clone,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match (
            project.stream_1.poll_update(cx),
            project.stream_2.poll_update(cx),
        ) {
            (Poll::Ready(value_1), Poll::Ready(value_2)) => {
                *project.prev_value_2 = Some(value_2.clone());
                Poll::Ready((Some(value_1), value_2))
            }
            (Poll::Ready(value_1), Poll::Pending) => Poll::Ready((
                Some(value_1),
                project
                    .prev_value_2
                    .as_ref()
                    .expect("should be initialized")
                    .clone(),
            )),
            (Poll::Pending, Poll::Ready(value_2)) => {
                *project.prev_value_2 = Some(value_2.clone());
                Poll::Ready((None, value_2))
            }
            (Poll::Pending, Poll::Pending) => Poll::Pending,
        }
    }
}

/// # ZipEvent signal's push timeout stream.
///
/// Created by the method [ZipEventSignal::push_timeout].
#[pin_project(project = ZipEventPushTimeoutProj)]
#[stream(timeout, item = (Option<U>, V))]
pub struct ZipEventPushTimeout<U, V, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = Option<U>>,
    PushT2: PushTimeoutStream<Item = V>,
    V: Clone,
{
    #[pin]
    stream_1: PushT1,
    #[pin]
    stream_2: PushT2,
    prev_value_2: Option<V>,
}
impl<U, V, PushT1, PushT2> PushTimeoutStream for ZipEventPushTimeout<U, V, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = Option<U>>,
    PushT2: PushTimeoutStream<Item = V>,
    V: Clone,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match (
            project.stream_1.poll_timeout(cx),
            project.stream_2.poll_timeout(cx),
        ) {
            (Poll::Ready(value_1), Poll::Ready(value_2)) => {
                *project.prev_value_2 = Some(value_2.clone());
                Poll::Ready((value_1, value_2))
            }
            (Poll::Ready(value_1), Poll::Pending) => Poll::Ready((
                value_1,
                project
                    .prev_value_2
                    .as_ref()
                    .expect("should be initialized")
                    .clone(),
            )),
            (Poll::Pending, Poll::Ready(value_2)) => {
                *project.prev_value_2 = Some(value_2.clone());
                Poll::Ready((None, value_2))
            }
            (Poll::Pending, Poll::Pending) => Poll::Pending,
        }
    }
}
