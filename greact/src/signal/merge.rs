use pin_project::pin_project;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use stream_proc_macros::stream;

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{signal::Signal, stream::push_timeout_stream::PushTimeoutStream};

/// Creates a [MergeSignal] that merges the two given signals.
pub fn merge<T, E1, E2>(signal_1: E1, signal_2: E2) -> MergeSignal<T, E1, E2>
where
    E1: Signal<T>,
    E2: Signal<T>,
    T: PartialEq + Clone,
{
    MergeSignal {
        signal_1,
        signal_2,
        phantom: PhantomData,
    }
}

/// # Merge signal.
///
/// An signal merging the two source signals.
/// It implements Signal trait, allowing to sample it synchronously
/// (creating a pull stream [MergePull]) or asynchronously
/// (creating a push stream [MergePush]).
pub struct MergeSignal<T, E1, E2>
where
    E1: Signal<T>,
    E2: Signal<T>,
    T: PartialEq + Clone,
{
    signal_1: E1,
    signal_2: E2,
    phantom: PhantomData<T>,
}
impl<T, E1, E2> Signal<T> for MergeSignal<T, E1, E2>
where
    E1: Signal<T>,
    E2: Signal<T>,
    T: PartialEq + Clone,
{
    type PullStream = MergePull<T, E1::PullStream, E2::PullStream>;
    type PushStream = MergePush<T, E1::PushStream, E2::PushStream>;
    type PushTimeoutStream = MergePushTimeout<T, E1::PushTimeoutStream, E2::PushTimeoutStream>;

    fn pull(self) -> Self::PullStream {
        let mut pull_stream_1 = self.signal_1.pull();
        let mut pull_stream_2 = self.signal_2.pull();
        let previous_value_1 = pull_stream_1.pick();
        let previous_value_2 = pull_stream_2.pick();
        let previous_value = previous_value_1.clone();

        MergePull {
            pull_stream_1,
            pull_stream_2,
            previous_value_1,
            previous_value_2,
            previous_value,
        }
    }

    fn push(self) -> Self::PushStream {
        let stream_1 = self.signal_1.push();
        let stream_2 = self.signal_2.push();

        MergePush { stream_1, stream_2 }
    }

    fn push_timeout(self, dur: std::time::Duration) -> Self::PushTimeoutStream {
        let stream_1 = self.signal_1.push_timeout(dur);
        let stream_2 = self.signal_2.push_timeout(dur);

        MergePushTimeout { stream_1, stream_2 }
    }
}

/// # Merge signal's pull stream.
///
/// Created by the method [MergeSignal::pull].
pub struct MergePull<T, Pull1, Pull2>
where
    Pull1: PullStream<Item = T>,
    Pull2: PullStream<Item = T>,
    T: PartialEq + Clone,
{
    pull_stream_1: Pull1,
    pull_stream_2: Pull2,
    previous_value_1: T,
    previous_value_2: T,
    previous_value: T,
}
impl<T, Pull1, Pull2> PullStream for MergePull<T, Pull1, Pull2>
where
    Pull1: PullStream<Item = T>,
    Pull2: PullStream<Item = T>,
    T: PartialEq + Clone,
{
    type Item = T;

    fn pick(&mut self) -> Self::Item {
        let value_1 = self.pull_stream_1.pick();
        let value_2 = self.pull_stream_2.pick();

        if value_1 != self.previous_value_1 {
            // value 1 has been updated
            self.previous_value_1 = value_1.clone();
            self.previous_value = value_1.clone();
            value_1
        } else if value_2 != self.previous_value_2 {
            // value 2 has been updated
            self.previous_value_2 = value_2.clone();
            self.previous_value = value_2.clone();
            value_2
        } else {
            // no value has been updated
            self.previous_value.clone()
        }
    }
}

/// # Merge signal's push stream.
///
/// Created by the method [MergeSignal::push].
#[pin_project(project = MergePushProj)]
#[stream(push, item = T)]
pub struct MergePush<T, Push1, Push2>
where
    Push1: PushStream<Item = T>,
    Push2: PushStream<Item = T>,
{
    #[pin]
    stream_1: Push1,
    #[pin]
    stream_2: Push2,
}
impl<T, Push1, Push2> PushStream for MergePush<T, Push1, Push2>
where
    Push1: PushStream<Item = T>,
    Push2: PushStream<Item = T>,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match (
            project.stream_1.poll_update(cx),
            project.stream_2.poll_update(cx),
        ) {
            // the first stream is updated
            (Poll::Ready(value), _) => Poll::Ready(value),
            // the second stream is updated
            (_, Poll::Ready(value)) => Poll::Ready(value),
            // any other cases wait
            _ => Poll::Pending,
        }
    }
}

/// # Merge signal's push timeout stream.
///
/// Created by the method [MergeSignal::push].
#[pin_project(project = MergePushTimeoutProj)]
#[stream(timeout, item = T)]
pub struct MergePushTimeout<T, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = T>,
    PushT2: PushTimeoutStream<Item = T>,
{
    #[pin]
    stream_1: PushT1,
    #[pin]
    stream_2: PushT2,
}
impl<T, PushT1, PushT2> PushTimeoutStream for MergePushTimeout<T, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = T>,
    PushT2: PushTimeoutStream<Item = T>,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match (
            project.stream_1.poll_timeout(cx),
            project.stream_2.poll_timeout(cx),
        ) {
            // the first stream is updated
            (Poll::Ready(value), _) => Poll::Ready(value),
            // the second stream is updated
            (_, Poll::Ready(value)) => Poll::Ready(value),
            // any other cases wait
            _ => Poll::Pending,
        }
    }
}
