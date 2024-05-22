use pin_project::pin_project;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use stream_proc_macros::stream;

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{signal::Signal, stream::push_timeout_stream::PushTimeoutStream};

/// Creates a [FoldSignal] that folds the function to the given signal.
pub fn fold<U, V, S, F>(initial: V, signal: S, function: F) -> FoldSignal<U, V, S, F>
where
    S: Signal<U>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    FoldSignal {
        signal,
        function,
        previous: initial,
        phantom: PhantomData,
    }
}

/// # Fold signal.
///
/// A signal folding the function to every value from the source signal.
/// It implements Signal trait, allowing to sample it synchronously
/// (creating a pull stream [FoldPull]) or asynchronously
/// (creating a push stream [FoldPush]).
pub struct FoldSignal<U, V, S, F>
where
    S: Signal<U>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    signal: S,
    function: F,
    previous: V,
    phantom: PhantomData<U>,
}
impl<U, V, S, F> Signal<V> for FoldSignal<U, V, S, F>
where
    S: Signal<U>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    type PullStream = FoldPull<U, V, S::PullStream, F>;
    type PushStream = FoldPush<U, V, S::PushStream, F>;
    type PushTimeoutStream = FoldPushTimeout<U, V, S::PushTimeoutStream, F>;

    fn pull(self) -> Self::PullStream {
        let pull_stream = self.signal.pull();
        FoldPull {
            pull_stream,
            function: self.function,
            previous: self.previous,
        }
    }

    fn push(self) -> Self::PushStream {
        let stream = self.signal.push();
        FoldPush {
            stream,
            function: self.function,
            previous: self.previous,
        }
    }

    fn push_timeout(self, dur: std::time::Duration) -> Self::PushTimeoutStream {
        let stream = self.signal.push_timeout(dur);
        FoldPushTimeout {
            stream,
            function: self.function,
            previous: self.previous,
        }
    }
}

/// # Fold signal's pull stream.
///
/// Created by the method [FoldSignal::pull].
pub struct FoldPull<U, V, Pull, F>
where
    Pull: PullStream<Item = U>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    pull_stream: Pull,
    function: F,
    previous: V,
}
impl<U, V, Pull, F> PullStream for FoldPull<U, V, Pull, F>
where
    Pull: PullStream<Item = U>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    type Item = V;

    fn pick(&mut self) -> Self::Item {
        let new_value =
            (self.function)(std::mem::take(&mut self.previous), self.pull_stream.pick());
        self.previous = new_value.clone();

        new_value
    }
}

/// # Fold signal's push stream.
///
/// Created by the method [FoldSignal::push].
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
            Poll::Ready(value) => Poll::Ready({
                let new_value = (project.function)(std::mem::take(project.previous), value);
                *project.previous = new_value.clone();
                new_value
            }),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// # Fold signal's push timeout stream.
///
/// Created by the method [FoldSignal::push_timeout].
#[pin_project(project = FoldPushTimeoutProj)]
#[stream(timeout, item = V)]
pub struct FoldPushTimeout<U, V, PushT, F>
where
    PushT: PushTimeoutStream<Item = U>,
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
    PushT: PushTimeoutStream<Item = U>,
    F: FnMut(V, U) -> V,
    V: Clone + Default,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_timeout(cx) {
            Poll::Ready(value) => Poll::Ready({
                let new_value = (project.function)(std::mem::take(project.previous), value);
                *project.previous = new_value.clone();
                new_value
            }),
            Poll::Pending => Poll::Pending,
        }
    }
}
