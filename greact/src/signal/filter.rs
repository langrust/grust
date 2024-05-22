use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use stream_proc_macros::stream;

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{signal::Signal, stream::push_timeout_stream::PushTimeoutStream};

/// Creates a [FilterSignal] that filters the given signal over a predicate.
pub fn filter<T, S, F>(initial_value: T, signal: S, predicate: F) -> FilterSignal<T, S, F>
where
    S: Signal<T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    FilterSignal {
        initial_value,
        signal,
        predicate,
    }
}

/// # Filter signal.
///
/// A predicate filtering the source signal.
/// It implements Signal trait, allowing to sample it synchronously
/// (creating a pull stream [FilterPull]) or asynchronously
/// (creating a push stream [FilterPush]).
pub struct FilterSignal<T, S, F>
where
    S: Signal<T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    initial_value: T,
    signal: S,
    predicate: F,
}
impl<T, S, F> Signal<T> for FilterSignal<T, S, F>
where
    S: Signal<T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    type PullStream = FilterPull<T, S::PullStream, F>;
    type PushStream = FilterPush<T, S::PushStream, F>;
    type PushTimeoutStream = FilterPushTimeout<T, S::PushTimeoutStream, F>;

    fn pull(self) -> Self::PullStream {
        let pull_stream = self.signal.pull();
        let predicate = self.predicate;
        let previous_value = self.initial_value;

        FilterPull {
            pull_stream,
            predicate,
            previous_value,
        }
    }

    fn push(self) -> Self::PushStream {
        let stream = self.signal.push();
        let predicate = self.predicate;
        let initial_value = Some(self.initial_value);

        FilterPush {
            stream,
            predicate,
            initial_value,
        }
    }

    fn push_timeout(self, dur: std::time::Duration) -> Self::PushTimeoutStream {
        let stream = self.signal.push_timeout(dur);
        let predicate = self.predicate;
        let initial_value = Some(self.initial_value);

        FilterPushTimeout {
            stream,
            predicate,
            initial_value,
        }
    }
}

/// # Filter signal's pull stream.
///
/// Created by the method [FilterSignal::pull].
pub struct FilterPull<T, Pull, F>
where
    Pull: PullStream<Item = T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    pull_stream: Pull,
    predicate: F,
    previous_value: T,
}
impl<T, Pull, F> PullStream for FilterPull<T, Pull, F>
where
    Pull: PullStream<Item = T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    type Item = T;

    fn pick(&mut self) -> Self::Item {
        let new_value = self.pull_stream.pick();
        if (self.predicate)(new_value.clone()) {
            self.previous_value = new_value.clone();
            new_value
        } else {
            self.previous_value.clone()
        }
    }
}

/// # Filter signal's push stream.
///
/// Created by the method [FilterSignal::push].
#[pin_project(project = FilterPushProj)]
#[stream(push, item = T)]
pub struct FilterPush<T, Push, F>
where
    Push: PushStream<Item = T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    #[pin]
    stream: Push,
    predicate: F,
    initial_value: Option<T>,
}
impl<T, Push, F> PushStream for FilterPush<T, Push, F>
where
    Push: PushStream<Item = T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();

        // initialize the stream when first value is filtered
        if let Some(initial_value) = project.initial_value.take() {
            match project.stream.poll_update(cx) {
                Poll::Ready(new_value) => {
                    if (project.predicate)(new_value.clone()) {
                        Poll::Ready(new_value)
                    } else {
                        Poll::Ready(initial_value)
                    }
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            match project.stream.poll_update(cx) {
                Poll::Ready(new_value) => {
                    if (project.predicate)(new_value.clone()) {
                        Poll::Ready(new_value)
                    } else {
                        Poll::Pending
                    }
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

/// # Filter signal's push timeout stream.
///
/// Created by the method [FilterSignal::push_timeout].
#[pin_project(project = FilterPushTimeoutProj)]
#[stream(timeout, item = T)]
pub struct FilterPushTimeout<T, PushT, F>
where
    PushT: PushTimeoutStream<Item = T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    #[pin]
    stream: PushT,
    predicate: F,
    initial_value: Option<T>,
}
impl<T, PushT, F> PushTimeoutStream for FilterPushTimeout<T, PushT, F>
where
    PushT: PushTimeoutStream<Item = T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();

        // initialize the stream when first value is filtered
        if let Some(initial_value) = project.initial_value.take() {
            match project.stream.poll_timeout(cx) {
                Poll::Ready(new_value) => {
                    if (project.predicate)(new_value.clone()) {
                        Poll::Ready(new_value)
                    } else {
                        Poll::Ready(initial_value)
                    }
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            match project.stream.poll_timeout(cx) {
                Poll::Ready(new_value) => {
                    if (project.predicate)(new_value.clone()) {
                        Poll::Ready(new_value)
                    } else {
                        Poll::Pending
                    }
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }
}
