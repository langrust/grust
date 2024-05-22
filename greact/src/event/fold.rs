use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;
use stream_proc_macros::stream;

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{event::Event, stream::push_timeout_stream::PushTimeoutStream};

/// Creates a [FoldEvent] that folds the function to the given event.
pub fn fold<U, V, E, F>(initial: V, event: E, function: F) -> FoldEvent<U, V, E, F>
where
    E: Event<U>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    FoldEvent {
        event,
        function,
        previous: initial,
        phantom: PhantomData,
    }
}

/// # Fold event.
///
/// An event folding the function to every value from the source event.
/// It implements Event trait, allowing to sample it synchronously
/// (creating a pull stream [FoldPull]), asynchronously
/// (creating a push stream [FoldPush]) or asynchronously with a
/// periodic timeout (creating a push timeout stream [FoldPushTimeout]).
pub struct FoldEvent<U, V, E, F>
where
    E: Event<U>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    event: E,
    function: F,
    previous: V,
    phantom: PhantomData<U>,
}
impl<U, V, E, F> Event<V> for FoldEvent<U, V, E, F>
where
    E: Event<U>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    type PullStream = FoldPull<U, V, E::PullStream, F>;
    type PushStream = FoldPush<U, V, E::PushStream, F>;
    type PushTimeoutStream = FoldPushTimeout<U, V, E::PushTimeoutStream, F>;

    fn pull(self) -> Self::PullStream {
        let pull_stream = self.event.pull();
        FoldPull {
            pull_stream,
            function: self.function,
            previous: self.previous,
        }
    }

    fn push(self) -> Self::PushStream {
        let stream = self.event.push();
        FoldPush {
            stream,
            function: self.function,
            previous: self.previous,
        }
    }

    fn push_timeout(self, dur: std::time::Duration) -> Self::PushTimeoutStream {
        let stream = self.event.push_timeout(dur);
        FoldPushTimeout {
            stream,
            function: self.function,
            previous: self.previous,
        }
    }
}

/// # Fold event's pull stream.
///
/// Created by the method [FoldEvent::pull].
pub struct FoldPull<U, V, Pull, F>
where
    Pull: PullStream<Item = Option<U>>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    pull_stream: Pull,
    function: F,
    previous: V,
}
impl<U, V, Pull, F> PullStream for FoldPull<U, V, Pull, F>
where
    Pull: PullStream<Item = Option<U>>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    type Item = Option<V>;

    fn pick(&mut self) -> Self::Item {
        self.pull_stream.pick().map(|value| {
            let new_value = (self.function)(std::mem::take(&mut self.previous), value);
            self.previous = new_value.clone();
            new_value
        })
    }
}

/// # Fold event's push stream.
///
/// Created by the method [FoldEvent::push].
#[pin_project(project = FoldPushProj)]
#[stream(push, item = V)]
pub struct FoldPush<U, V, Push, F>
where
    Push: PushStream<Item = U>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    #[pin]
    stream: Push,
    function: F,
    previous: V,
}
impl<U, V, Push, F> PushStream for FoldPush<U, V, Push, F>
where
    Push: PushStream<Item = U>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_update(cx) {
            Poll::Ready(value) => {
                let new_value = (project.function)(std::mem::take(project.previous), value);
                *project.previous = new_value.clone();
                Poll::Ready(new_value)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// # Fold event's push timeout stream.
///
/// Created by the method [FoldEvent::push_timeout].
#[pin_project(project = FoldPushTimeoutProj)]
#[stream(timeout, item = Option<V>)]
pub struct FoldPushTimeout<U, V, PushT, F>
where
    PushT: PushTimeoutStream<Item = Option<U>>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    #[pin]
    stream: PushT,
    function: F,
    previous: V,
}
impl<U, V, PushT, F> PushTimeoutStream for FoldPushTimeout<U, V, PushT, F>
where
    PushT: PushTimeoutStream<Item = Option<U>>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_timeout(cx) {
            Poll::Ready(option) => Poll::Ready(option.map(|value| {
                let new_value = (project.function)(std::mem::take(project.previous), value);
                *project.previous = new_value.clone();
                new_value
            })),
            Poll::Pending => Poll::Pending,
        }
    }
}
