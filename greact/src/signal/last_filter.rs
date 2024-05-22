use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use stream_proc_macros::stream;

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{signal::Signal, stream::push_timeout_stream::PushTimeoutStream};

/// Creates a [LastFilterSignal] that filters the given signal with another signal.
pub fn last_filter<T, S1, S2>(
    initial_value: T,
    signal_values: S1,
    signal_filter: S2,
) -> LastFilterSignal<T, S1, S2>
where
    S1: Signal<T>,
    S2: Signal<bool>,
    T: Clone,
{
    LastFilterSignal {
        initial_value,
        signal_values,
        signal_filter,
    }
}

/// # LastFilter signal.
///
/// A signal filtering the source signal.
/// It implements Signal trait, allowing to sample it synchronously
/// (creating a pull stream [LastFilterPull]) or asynchronously
/// (creating a push stream [LastFilterPush]).
pub struct LastFilterSignal<T, S1, S2>
where
    S1: Signal<T>,
    S2: Signal<bool>,
    T: Clone,
{
    initial_value: T,
    signal_values: S1,
    signal_filter: S2,
}
impl<T, S1, S2> Signal<T> for LastFilterSignal<T, S1, S2>
where
    S1: Signal<T>,
    S2: Signal<bool>,
    T: Clone,
{
    type PullStream = LastFilterPull<T, S1::PullStream, S2::PullStream>;
    type PushStream = LastFilterPush<T, S1::PushStream, S2::PushStream>;
    type PushTimeoutStream = LastFilterPushTimeout<T, S1::PushTimeoutStream, S2::PushTimeoutStream>;

    fn pull(self) -> Self::PullStream {
        let pull_stream_values = self.signal_values.pull();
        let pull_stream_filter = self.signal_filter.pull();
        let previous_value = self.initial_value;

        LastFilterPull {
            pull_stream_values,
            pull_stream_filter,
            previous_value,
        }
    }

    fn push(self) -> Self::PushStream {
        let stream_values = self.signal_values.push();
        let stream_filter = self.signal_filter.push();
        let initial_value = Some(self.initial_value);

        LastFilterPush {
            stream_values,
            stream_filter,
            initial_value,
        }
    }

    fn push_timeout(self, dur: std::time::Duration) -> Self::PushTimeoutStream {
        let stream_values = self.signal_values.push_timeout(dur);
        let stream_filter = self.signal_filter.push_timeout(dur);
        let initial_value = Some(self.initial_value);

        LastFilterPushTimeout {
            stream_values,
            stream_filter,
            initial_value,
        }
    }
}

/// # LastFilter signal's pull stream.
///
/// Created by the method [LastFilterSignal::pull].
pub struct LastFilterPull<T, Pull1, Pull2>
where
    Pull1: PullStream<Item = T>,
    Pull2: PullStream<Item = bool>,
    T: Clone,
{
    pull_stream_values: Pull1,
    pull_stream_filter: Pull2,
    previous_value: T,
}
impl<T, Pull1, Pull2> PullStream for LastFilterPull<T, Pull1, Pull2>
where
    Pull1: PullStream<Item = T>,
    Pull2: PullStream<Item = bool>,
    T: Clone,
{
    type Item = T;

    fn pick(&mut self) -> Self::Item {
        let filter = self.pull_stream_filter.pick();
        if filter {
            let new_value = self.pull_stream_values.pick();
            self.previous_value = new_value.clone();
            new_value
        } else {
            self.previous_value.clone()
        }
    }
}

/// # LastFilter signal's push stream.
///
/// Created by the method [LastFilterSignal::push].
#[pin_project(project = LastFilterPushProj)]
#[stream(push, item = T)]
pub struct LastFilterPush<T, Push1, Push2>
where
    Push1: PushStream<Item = T>,
    Push2: PushStream<Item = bool>,
    T: Clone,
{
    #[pin]
    stream_values: Push1,
    #[pin]
    stream_filter: Push2,
    initial_value: Option<T>,
}
impl<T, Push1, Push2> PushStream for LastFilterPush<T, Push1, Push2>
where
    Push1: PushStream<Item = T>,
    Push2: PushStream<Item = bool>,
    T: Clone,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();

        // initialize the stream when first value is filtered
        if let Some(initial_value) = project.initial_value.take() {
            match project.stream_filter.poll_update(cx) {
                Poll::Ready(filter) => {
                    if filter {
                        project.stream_values.poll_update(cx)
                    } else {
                        Poll::Ready(initial_value)
                    }
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            match project.stream_filter.poll_update(cx) {
                Poll::Ready(filter) => {
                    if filter {
                        project.stream_values.poll_update(cx)
                    } else {
                        Poll::Pending
                    }
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

/// # LastFilter signal's push stream.
///
/// Created by the method [LastFilterSignal::push].
#[pin_project(project = LastFilterPushTimeoutProj)]
#[stream(timeout, item = T)]
pub struct LastFilterPushTimeout<T, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = T>,
    PushT2: PushTimeoutStream<Item = bool>,
    T: Clone,
{
    #[pin]
    stream_values: PushT1,
    #[pin]
    stream_filter: PushT2,
    initial_value: Option<T>,
}
impl<T, PushT1, PushT2> PushTimeoutStream for LastFilterPushTimeout<T, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = T>,
    PushT2: PushTimeoutStream<Item = bool>,
    T: Clone,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();

        // initialize the stream when first value is filtered
        if let Some(initial_value) = project.initial_value.take() {
            match project.stream_filter.poll_timeout(cx) {
                Poll::Ready(filter) => {
                    if filter {
                        project.stream_values.poll_timeout(cx)
                    } else {
                        Poll::Ready(initial_value)
                    }
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            match project.stream_filter.poll_timeout(cx) {
                Poll::Ready(filter) => {
                    if filter {
                        project.stream_values.poll_timeout(cx)
                    } else {
                        Poll::Pending
                    }
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }
}
