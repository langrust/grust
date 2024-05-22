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

/// Creates a [MergeEvent] that merges the two given events.
pub fn merge<T, E1, E2>(event_1: E1, event_2: E2) -> MergeEvent<T, E1, E2>
where
    E1: Event<T>,
    E2: Event<T>,
{
    MergeEvent {
        event_1,
        event_2,
        phantom: PhantomData,
    }
}

/// # Merge event.
///
/// An event merging the two source events.
/// It implements Event trait, allowing to sample it synchronously
/// (creating a pull stream [MergePull]) or asynchronously
/// (creating a push stream [MergePush]).
pub struct MergeEvent<T, E1, E2>
where
    E1: Event<T>,
    E2: Event<T>,
{
    event_1: E1,
    event_2: E2,
    phantom: PhantomData<T>,
}
impl<T, E1, E2> Event<T> for MergeEvent<T, E1, E2>
where
    E1: Event<T>,
    E2: Event<T>,
{
    type PullStream = MergePull<T, E1::PullStream, E2::PullStream>;
    type PushStream = MergePush<T, E1::PushStream, E2::PushStream>;
    type PushTimeoutStream = MergePushTimeout<T, E1::PushTimeoutStream, E2::PushTimeoutStream>;

    fn pull(self) -> Self::PullStream {
        let pull_stream_1 = self.event_1.pull();
        let pull_stream_2 = self.event_2.pull();

        MergePull {
            pull_stream_1,
            pull_stream_2,
        }
    }

    fn push(self) -> Self::PushStream {
        let previous_producer = true;
        let stream_1 = self.event_1.push();
        let stream_2 = self.event_2.push();

        MergePush {
            previous_producer,
            stream_1,
            stream_2,
        }
    }

    fn push_timeout(self, dur: Duration) -> Self::PushTimeoutStream {
        let previous_producer = true;
        let stream_1 = self.event_1.push_timeout(dur);
        let stream_2 = self.event_2.push_timeout(dur);

        MergePushTimeout {
            previous_producer,
            stream_1,
            stream_2,
        }
    }
}

/// # Merge event's pull stream.
///
/// Created by the method [MergeEvent::pull].
pub struct MergePull<T, Pull1, Pull2>
where
    Pull1: PullStream<Item = Option<T>>,
    Pull2: PullStream<Item = Option<T>>,
{
    pull_stream_1: Pull1,
    pull_stream_2: Pull2,
}
impl<T, Pull1, Pull2> PullStream for MergePull<T, Pull1, Pull2>
where
    Pull1: PullStream<Item = Option<T>>,
    Pull2: PullStream<Item = Option<T>>,
{
    type Item = Option<T>;

    fn pick(&mut self) -> Self::Item {
        let option_1 = self.pull_stream_1.pick();
        let option_2 = self.pull_stream_2.pick();

        option_1.or(option_2)
    }
}

/// # Merge event's push stream.
///
/// Created by the method [MergeEvent::push].
#[pin_project(project = MergePushProj)]
#[stream(push, item = T)]
pub struct MergePush<T, Push1, Push2>
where
    Push1: PushStream<Item = T>,
    Push2: PushStream<Item = T>,
{
    previous_producer: bool,
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
            // the first stream is the new producer
            (Poll::Ready(value), _) => {
                *project.previous_producer = true;
                Poll::Ready(value)
            }
            // the second stream is the new producer
            (_, Poll::Ready(value)) => {
                *project.previous_producer = false;
                Poll::Ready(value)
            }
            // any other cases wait
            _ => Poll::Pending,
        }
    }
}

/// # Merge event's push timeout stream.
///
/// Created by the method [MergeEvent::push_timeout].
#[pin_project(project = MergePushTimeoutProj)]
#[stream(timeout, item = Option<T>)]
pub struct MergePushTimeout<T, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = Option<T>>,
    PushT2: PushTimeoutStream<Item = Option<T>>,
{
    previous_producer: bool,
    #[pin]
    stream_1: PushT1,
    #[pin]
    stream_2: PushT2,
}
impl<T, PushT1, PushT2> PushTimeoutStream for MergePushTimeout<T, PushT1, PushT2>
where
    PushT1: PushTimeoutStream<Item = Option<T>>,
    PushT2: PushTimeoutStream<Item = Option<T>>,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match (
            project.stream_1.poll_timeout(cx),
            project.stream_2.poll_timeout(cx),
        ) {
            // the first stream is the new producer
            (Poll::Ready(Some(value)), _) => {
                *project.previous_producer = true;
                Poll::Ready(Some(value))
            }
            // the second stream is the new producer
            (_, Poll::Ready(Some(value))) => {
                *project.previous_producer = false;
                Poll::Ready(Some(value))
            }
            // the first stream was the previous producer and timeouts
            (Poll::Ready(None), _) if *project.previous_producer => Poll::Ready(None),
            // the second stream was the previous producer and timeouts
            (_, Poll::Ready(None)) if !*project.previous_producer => Poll::Ready(None),
            // any other cases wait
            _ => Poll::Pending,
        }
    }
}
