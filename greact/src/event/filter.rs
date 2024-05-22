use pin_project::pin_project;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use stream_proc_macros::stream;

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{event::Event, stream::push_timeout_stream::PushTimeoutStream};

/// Creates a [FilterEvent] that filters the given event over a predicate.
pub fn filter<T, E, F>(event: E, predicate: F) -> FilterEvent<T, E, F>
where
    E: Event<T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    FilterEvent {
        event,
        predicate,
        phantom: PhantomData,
    }
}

/// # Filter event.
///
/// A predicate filtering the source event.
/// It implements Event trait, allowing to sample it synchronously
/// (creating a pull stream [FilterPull]), asynchronously
/// (creating a push stream [FilterPush]) or asynchronously with
/// periodic timeout (creating a push timeout stream [FilterPushTimeout]).
pub struct FilterEvent<T, E, F>
where
    E: Event<T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    event: E,
    predicate: F,
    phantom: PhantomData<T>,
}
impl<T, E, F> Event<T> for FilterEvent<T, E, F>
where
    E: Event<T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    type PullStream = FilterPull<T, E::PullStream, F>;
    type PushStream = FilterPush<T, E::PushStream, F>;
    type PushTimeoutStream = FilterPushTimeout<T, E::PushTimeoutStream, F>;

    fn pull(self) -> Self::PullStream {
        let pull_stream = self.event.pull();
        let predicate = self.predicate;

        FilterPull {
            pull_stream,
            predicate,
        }
    }

    fn push(self) -> Self::PushStream {
        let stream = self.event.push();
        let predicate = self.predicate;

        FilterPush { stream, predicate }
    }

    fn push_timeout(self, dur: std::time::Duration) -> Self::PushTimeoutStream {
        let stream = self.event.push_timeout(dur);
        let predicate = self.predicate;

        FilterPushTimeout { stream, predicate }
    }
}

/// # Filter event's pull stream.
///
/// Created by the method [FilterEvent::pull].
pub struct FilterPull<T, Pull, F>
where
    Pull: PullStream<Item = Option<T>>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    pull_stream: Pull,
    predicate: F,
}
impl<T, Pull, F> PullStream for FilterPull<T, Pull, F>
where
    Pull: PullStream<Item = Option<T>>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    type Item = Option<T>;

    fn pick(&mut self) -> Self::Item {
        let option_value = self.pull_stream.pick();

        option_value.filter(|value| (self.predicate)(value.clone()))
    }
}

/// # Filter event's push stream.
///
/// Created by the method [FilterEvent::push].
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
}
impl<T, Push, F> PushStream for FilterPush<T, Push, F>
where
    Push: PushStream<Item = T>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_update(cx) {
            Poll::Ready(value) => {
                if (project.predicate)(value.clone()) {
                    Poll::Ready(value)
                } else {
                    Poll::Pending
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// # Filter event's push timeout stream.
///
/// Created by the method [FilterEvent::push_timeout].
#[pin_project(project = FilterPushTimeoutProj)]
#[stream(timeout, item = Option<T>)]
pub struct FilterPushTimeout<T, PushT, F>
where
    PushT: PushTimeoutStream<Item = Option<T>>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    #[pin]
    stream: PushT,
    predicate: F,
}
impl<T, PushT, F> PushTimeoutStream for FilterPushTimeout<T, PushT, F>
where
    PushT: PushTimeoutStream<Item = Option<T>>,
    F: FnMut(T) -> bool,
    T: Clone,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_timeout(cx) {
            Poll::Ready(Some(value)) => {
                if (project.predicate)(value.clone()) {
                    Poll::Ready(Some(value))
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
