use pin_project::pin_project;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use stream_proc_macros::stream;

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{signal::Signal, stream::push_timeout_stream::PushTimeoutStream};

/// Creates a [ZipSignal] that zips the two given signals.
pub fn zip<U, V, S1, S2>(signal_1: S1, signal_2: S2) -> ZipSignal<U, V, S1, S2>
where
    S1: Signal<U>,
    S2: Signal<V>,
    U: Clone,
    V: Clone,
{
    ZipSignal {
        signal_1,
        signal_2,
        phantom: PhantomData,
    }
}

/// # Zip signal.
///
/// A signal zipping two signals.
/// It implements Signal trait, allowing to sample it synchronously
/// (creating a pull stream [ZipPull]) or asynchronously
/// (creating a push stream [ZipPush]).
pub struct ZipSignal<U, V, S1, S2>
where
    S1: Signal<U>,
    S2: Signal<V>,
    U: Clone,
    V: Clone,
{
    signal_1: S1,
    signal_2: S2,
    phantom: PhantomData<(U, V)>,
}
impl<U, V, S1, S2> Signal<(U, V)> for ZipSignal<U, V, S1, S2>
where
    S1: Signal<U>,
    S2: Signal<V>,
    U: Clone,
    V: Clone,
{
    type PullStream = ZipPull<U, V, S1::PullStream, S2::PullStream>;
    type PushStream = ZipPush<U, V, S1::PushStream, S2::PushStream>;
    type PushTimeoutStream = ZipPushTimeout<U, V, S1::PushTimeoutStream, S2::PushTimeoutStream>;

    fn pull(self) -> Self::PullStream {
        let pull_stream_1 = self.signal_1.pull();
        let pull_stream_2 = self.signal_2.pull();
        ZipPull {
            pull_stream_1,
            pull_stream_2,
        }
    }

    fn push(self) -> Self::PushStream {
        let stream_1 = self.signal_1.push();
        let prev_value_1 = None;
        let stream_2 = self.signal_2.push();
        let prev_value_2 = None;
        ZipPush {
            stream_1,
            prev_value_1,
            stream_2,
            prev_value_2,
        }
    }

    fn push_timeout(self, dur: std::time::Duration) -> Self::PushTimeoutStream {
        let stream_1 = self.signal_1.push_timeout(dur);
        let prev_value_1 = None;
        let stream_2 = self.signal_2.push_timeout(dur);
        let prev_value_2 = None;
        ZipPushTimeout {
            stream_1,
            prev_value_1,
            stream_2,
            prev_value_2,
        }
    }
}

/// # Zip signal's pull stream.
///
/// Created by the method [ZipSignal::pull].
pub struct ZipPull<U, V, Pull1, Pull2>
where
    Pull1: PullStream<Item = U>,
    Pull2: PullStream<Item = V>,
    U: Clone,
    V: Clone,
{
    pull_stream_1: Pull1,
    pull_stream_2: Pull2,
}
impl<U, V, Pull1, Pull2> PullStream for ZipPull<U, V, Pull1, Pull2>
where
    Pull1: PullStream<Item = U>,
    Pull2: PullStream<Item = V>,
    U: Clone,
    V: Clone,
{
    type Item = (U, V);

    fn pick(&mut self) -> Self::Item {
        (self.pull_stream_1.pick(), self.pull_stream_2.pick())
    }
}

/// # Zip signal's push stream.
///
/// Created by the method [ZipSignal::push].
#[pin_project(project = ZipPushProj)]
#[stream(push, item = (U, V))]
pub struct ZipPush<U, V, Push1, Push2>
where
    Push1: PushStream<Item = U>,
    Push2: PushStream<Item = V>,
    U: Clone,
    V: Clone,
{
    #[pin]
    stream_1: Push1,
    prev_value_1: Option<U>,
    #[pin]
    stream_2: Push2,
    prev_value_2: Option<V>,
}
impl<U, V, Push1, Push2> PushStream for ZipPush<U, V, Push1, Push2>
where
    Push1: PushStream<Item = U>,
    Push2: PushStream<Item = V>,
    U: Clone,
    V: Clone,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match (
            project.stream_1.poll_update(cx),
            project.stream_2.poll_update(cx),
        ) {
            (Poll::Ready(value_1), Poll::Ready(value_2)) => {
                *project.prev_value_1 = Some(value_1.clone());
                *project.prev_value_2 = Some(value_2.clone());
                Poll::Ready((value_1, value_2))
            }
            (Poll::Ready(value_1), Poll::Pending) => {
                *project.prev_value_1 = Some(value_1.clone());
                Poll::Ready((
                    value_1,
                    project
                        .prev_value_2
                        .as_ref()
                        .expect("should be initialized")
                        .clone(),
                ))
            }
            (Poll::Pending, Poll::Ready(value_2)) => {
                *project.prev_value_2 = Some(value_2.clone());
                Poll::Ready((
                    project
                        .prev_value_1
                        .as_ref()
                        .expect("should be initialized")
                        .clone(),
                    value_2,
                ))
            }
            (Poll::Pending, Poll::Pending) => Poll::Pending,
        }
    }
}

/// # Zip signal's push timeout stream.
///
/// Created by the method [ZipSignal::push_timeout].
#[pin_project(project = ZipPushTimeoutProj)]
#[stream(timeout, item = (U, V))]
pub struct ZipPushTimeout<U, V, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = U>,
    PushT2: PushTimeoutStream<Item = V>,
    U: Clone,
    V: Clone,
{
    #[pin]
    stream_1: PushT1,
    prev_value_1: Option<U>,
    #[pin]
    stream_2: PushT2,
    prev_value_2: Option<V>,
}
impl<U, V, PushT1, PushT2> PushTimeoutStream for ZipPushTimeout<U, V, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = U>,
    PushT2: PushTimeoutStream<Item = V>,
    U: Clone,
    V: Clone,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match (
            project.stream_1.poll_timeout(cx),
            project.stream_2.poll_timeout(cx),
        ) {
            (Poll::Ready(value_1), Poll::Ready(value_2)) => {
                *project.prev_value_1 = Some(value_1.clone());
                *project.prev_value_2 = Some(value_2.clone());
                Poll::Ready((value_1, value_2))
            }
            (Poll::Ready(value_1), Poll::Pending) => {
                *project.prev_value_1 = Some(value_1.clone());
                Poll::Ready((
                    value_1,
                    project
                        .prev_value_2
                        .as_ref()
                        .expect("should be initialized")
                        .clone(),
                ))
            }
            (Poll::Pending, Poll::Ready(value_2)) => {
                *project.prev_value_2 = Some(value_2.clone());
                Poll::Ready((
                    project
                        .prev_value_1
                        .as_ref()
                        .expect("should be initialized")
                        .clone(),
                    value_2,
                ))
            }
            (Poll::Pending, Poll::Pending) => Poll::Pending,
        }
    }
}
