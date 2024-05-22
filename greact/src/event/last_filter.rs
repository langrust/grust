use pin_project::pin_project;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use stream_proc_macros::stream;

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{event::Event, stream::push_timeout_stream::PushTimeoutStream};

/// Creates a [LastFilterEvent] that filters the given event with another event.
pub fn last_filter<T, E1, E2>(event_values: E1, event_filter: E2) -> LastFilterEvent<T, E1, E2>
where
    E1: Event<T>,
    E2: Event<bool>,
{
    LastFilterEvent {
        event_values,
        event_filter,
        phantom: PhantomData,
    }
}

/// # Filter event.
///
/// An event filtering the source event.
/// It implements Event trait, allowing to sample it synchronously
/// (creating a pull stream [LastFilterPull]) or asynchronously
/// (creating a push stream [LastFilterPush]).
pub struct LastFilterEvent<T, E1, E2>
where
    E1: Event<T>,
    E2: Event<bool>,
{
    event_values: E1,
    event_filter: E2,
    phantom: PhantomData<T>,
}
impl<T, E1, E2> Event<T> for LastFilterEvent<T, E1, E2>
where
    E1: Event<T>,
    E2: Event<bool>,
{
    type PullStream = LastFilterPull<T, E1::PullStream, E2::PullStream>;
    type PushStream = LastFilterPush<T, E1::PushStream, E2::PushStream>;
    type PushTimeoutStream = LastFilterPushTimeout<T, E1::PushTimeoutStream, E2::PushTimeoutStream>;

    fn pull(self) -> Self::PullStream {
        let pull_stream_values = self.event_values.pull();
        let pull_stream_filter = self.event_filter.pull();

        LastFilterPull {
            pull_stream_values,
            pull_stream_filter,
        }
    }

    fn push(self) -> Self::PushStream {
        let stream_values = self.event_values.push();
        let stream_filter = self.event_filter.push();

        LastFilterPush {
            stream_values,
            stream_filter,
        }
    }

    fn push_timeout(self, dur: std::time::Duration) -> Self::PushTimeoutStream {
        let stream_values = self.event_values.push_timeout(dur);
        let stream_filter = self.event_filter.push_timeout(dur);

        LastFilterPushTimeout {
            stream_values,
            stream_filter,
        }
    }
}

/// # LastFilter event's pull stream.
///
/// Created by the method [LastFilterEvent::pull].
pub struct LastFilterPull<T, Pull1, Pull2>
where
    Pull1: PullStream<Item = Option<T>>,
    Pull2: PullStream<Item = Option<bool>>,
{
    pull_stream_values: Pull1,
    pull_stream_filter: Pull2,
}
impl<T, Pull1, Pull2> PullStream for LastFilterPull<T, Pull1, Pull2>
where
    Pull1: PullStream<Item = Option<T>>,
    Pull2: PullStream<Item = Option<bool>>,
{
    type Item = Option<T>;

    fn pick(&mut self) -> Self::Item {
        let option_filter = self.pull_stream_filter.pick();

        option_filter
            .map(|filter| {
                if filter {
                    self.pull_stream_values.pick()
                } else {
                    None
                }
            })
            .flatten()
    }
}

/// # LastFilter event's push stream.
///
/// Created by the method [LastFilterEvent::push].
#[pin_project(project = LastFilterPushProj)]
#[stream(push, item = T)]
pub struct LastFilterPush<T, Push1, Push2>
where
    Push1: PushStream<Item = T>,
    Push2: PushStream<Item = bool>,
{
    #[pin]
    stream_values: Push1,
    #[pin]
    stream_filter: Push2,
}
impl<T, Push1, Push2> PushStream for LastFilterPush<T, Push1, Push2>
where
    Push1: PushStream<Item = T>,
    Push2: PushStream<Item = bool>,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
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

/// # LastFilter event's push timeout stream.
///
/// Created by the method [LastFilterEvent::push_timeout].
#[pin_project(project = LastFilterPushTimeoutProj)]
#[stream(timeout, item = Option<T>)]
pub struct LastFilterPushTimeout<T, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = Option<T>>,
    PushT2: PushTimeoutStream<Item = Option<bool>>,
{
    #[pin]
    stream_values: PushT1,
    #[pin]
    stream_filter: PushT2,
}
impl<T, PushT1, PushT2> PushTimeoutStream for LastFilterPushTimeout<T, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = Option<T>>,
    PushT2: PushTimeoutStream<Item = Option<bool>>,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream_filter.poll_timeout(cx) {
            Poll::Ready(Some(filter)) => {
                if filter {
                    project.stream_values.poll_timeout(cx)
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
