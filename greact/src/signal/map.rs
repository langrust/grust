use pin_project::pin_project;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use stream_proc_macros::stream;

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{signal::Signal, stream::push_timeout_stream::PushTimeoutStream};

/// Creates a [MapSignal] that maps the function to the given signal.
pub fn map<U, V, S, F>(signal: S, function: F) -> MapSignal<U, V, S, F>
where
    S: Signal<U>,
    F: FnMut(U) -> V,
{
    MapSignal {
        signal,
        function,
        phantom: PhantomData,
    }
}

/// # Map signal.
///
/// A signal mapping the function to every value from the source signal.
/// It implements Signal trait, allowing to sample it synchronously
/// (creating a pull stream [MapPull]) or asynchronously
/// (creating a push stream [MapPush]).
pub struct MapSignal<U, V, S, F>
where
    S: Signal<U>,
    F: FnMut(U) -> V,
{
    signal: S,
    function: F,
    phantom: PhantomData<(U, V)>,
}
impl<U, V, S, F> Signal<V> for MapSignal<U, V, S, F>
where
    S: Signal<U>,
    F: FnMut(U) -> V,
{
    type PullStream = MapPull<U, V, S::PullStream, F>;
    type PushStream = MapPush<U, V, S::PushStream, F>;
    type PushTimeoutStream = MapPushTimeout<U, V, S::PushTimeoutStream, F>;

    fn pull(self) -> Self::PullStream {
        let pull_stream = self.signal.pull();
        MapPull {
            pull_stream,
            function: self.function,
        }
    }

    fn push(self) -> Self::PushStream {
        let stream = self.signal.push();
        MapPush {
            stream,
            function: self.function,
        }
    }

    fn push_timeout(self, dur: std::time::Duration) -> Self::PushTimeoutStream {
        let stream = self.signal.push_timeout(dur);
        MapPushTimeout {
            stream,
            function: self.function,
        }
    }
}

/// # Map signal's pull stream.
///
/// Created by the method [MapSignal::pull].
pub struct MapPull<U, V, Pull, F>
where
    Pull: PullStream<Item = U>,
    F: FnMut(U) -> V,
{
    pull_stream: Pull,
    function: F,
}
impl<U, V, Pull, F> PullStream for MapPull<U, V, Pull, F>
where
    Pull: PullStream<Item = U>,
    F: FnMut(U) -> V,
{
    type Item = V;

    fn pick(&mut self) -> Self::Item {
        (self.function)(self.pull_stream.pick())
    }
}

/// # Map signal's push stream.
///
/// Created by the method [MapSignal::push].
#[pin_project(project = MapPushProj)]
#[stream(push, item = V)]
pub struct MapPush<U, V, Push, F>
where
    Push: PushStream<Item = U>,
    F: FnMut(U) -> V,
{
    #[pin]
    stream: Push,
    function: F,
}
impl<U, V, Push, F> PushStream for MapPush<U, V, Push, F>
where
    Push: PushStream<Item = U>,
    F: FnMut(U) -> V,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_update(cx) {
            Poll::Ready(value) => Poll::Ready((project.function)(value)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// # Map signal's push timeout stream.
///
/// Created by the method [MapSignal::push_timeout].
#[pin_project(project = MapPushTimeoutProj)]
#[stream(timeout, item = V)]
pub struct MapPushTimeout<U, V, PushT, F>
where
    PushT: PushTimeoutStream<Item = U>,
    F: FnMut(U) -> V,
{
    #[pin]
    stream: PushT,
    function: F,
}
impl<U, V, PushT, F> PushTimeoutStream for MapPushTimeout<U, V, PushT, F>
where
    PushT: PushTimeoutStream<Item = U>,
    F: FnMut(U) -> V,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_timeout(cx) {
            Poll::Ready(value) => Poll::Ready((project.function)(value)),
            Poll::Pending => Poll::Pending,
        }
    }
}
