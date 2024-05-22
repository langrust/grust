use pin_project::pin_project;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use stream_proc_macros::stream;

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{event::Event, stream::push_timeout_stream::PushTimeoutStream};

/// Creates a [MapEvent] that maps the function to the given event.
pub fn map<U, V, E, F>(event: E, function: F) -> MapEvent<U, V, E, F>
where
    E: Event<U>,
    F: FnMut(U) -> V,
{
    MapEvent {
        event,
        function,
        phantom: PhantomData,
    }
}

/// # Map event.
///
/// An event mapping the function to every value from the source event.
/// It implements Event trait, allowing to sample it synchronously
/// (creating a pull stream [MapPull]) or asynchronously
/// (creating a push stream [MapPush]).
pub struct MapEvent<U, V, E, F>
where
    E: Event<U>,
    F: FnMut(U) -> V,
{
    event: E,
    function: F,
    phantom: PhantomData<(U, V)>,
}
impl<U, V, E, F> Event<V> for MapEvent<U, V, E, F>
where
    E: Event<U>,
    F: FnMut(U) -> V,
{
    type PullStream = MapPull<U, V, E::PullStream, F>;
    type PushStream = MapPush<U, V, E::PushStream, F>;
    type PushTimeoutStream = MapPushTimeout<U, V, E::PushTimeoutStream, F>;

    fn pull(self) -> Self::PullStream {
        let pull_stream = self.event.pull();
        MapPull {
            pull_stream,
            function: self.function,
        }
    }

    fn push(self) -> Self::PushStream {
        let stream = self.event.push();
        MapPush {
            stream,
            function: self.function,
        }
    }

    fn push_timeout(self, dur: Duration) -> Self::PushTimeoutStream {
        let stream = self.event.push_timeout(dur);
        MapPushTimeout {
            stream,
            function: self.function,
        }
    }
}

/// # Map event's pull stream.
///
/// Created by the method [MapEvent::pull].
pub struct MapPull<U, V, Pull, F>
where
    Pull: PullStream<Item = Option<U>>,
    F: FnMut(U) -> V,
{
    pull_stream: Pull,
    function: F,
}
impl<U, V, Pull, F> PullStream for MapPull<U, V, Pull, F>
where
    Pull: PullStream<Item = Option<U>>,
    F: FnMut(U) -> V,
{
    type Item = Option<V>;

    fn pick(&mut self) -> Self::Item {
        self.pull_stream.pick().map(|value| (self.function)(value))
    }
}

/// # Map event's push stream.
///
/// Created by the method [MapEvent::push].
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

/// # Map event's push timeout stream.
///
/// Created by the method [MapEvent::push_timeout].
#[pin_project(project = MapPushTimeoutProj)]
#[stream(timeout, item = Option<V>)]
pub struct MapPushTimeout<U, V, PushT, F>
where
    PushT: PushTimeoutStream<Item = Option<U>>,
    F: FnMut(U) -> V,
{
    #[pin]
    stream: PushT,
    function: F,
}
impl<U, V, PushT, F> PushTimeoutStream for MapPushTimeout<U, V, PushT, F>
where
    PushT: PushTimeoutStream<Item = Option<U>>,
    F: FnMut(U) -> V,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_timeout(cx) {
            Poll::Ready(option) => Poll::Ready(option.map(project.function)),
            Poll::Pending => Poll::Pending,
        }
    }
}
