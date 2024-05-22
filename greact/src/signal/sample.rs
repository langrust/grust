use futures::Stream;
use pin_project::pin_project;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use stream_proc_macros::stream;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use crate::{
    event::Event,
    signal::Signal,
    stream::{
        pull_stream::PullStream, push_stream::PushStream, push_timeout_stream::PushTimeoutStream,
    },
};

/// Creates a [SampleSignal] that samples the given signal at a given period.
pub fn sample<U, S>(signal: S, period: Duration) -> SampleSignal<U, S>
where
    S: Signal<U>,
    U: Clone,
{
    SampleSignal {
        signal,
        period,
        phantom: PhantomData,
    }
}

/// # Sample signal.
///
/// An event sampling the source signal at the period.
/// It implements Event trait, allowing to sample it synchronously
/// (creating a pull stream [SamplePull]) or asynchronously
/// (creating a push stream [SamplePush]).
///
/// The synchronous sampling is unimplemented: requires clocks.
pub struct SampleSignal<U, S>
where
    S: Signal<U>,
    U: Clone,
{
    signal: S,
    period: Duration,
    phantom: PhantomData<U>,
}
impl<U, S> Event<U> for SampleSignal<U, S>
where
    S: Signal<U>,
    U: Clone,
{
    type PullStream = SamplePull<U, S::PullStream>;
    type PushStream = SamplePush<U, S::PushStream>;
    type PushTimeoutStream = SamplePushTimeout<U, S::PushTimeoutStream>;

    fn pull(self) -> Self::PullStream {
        unimplemented!()
    }

    fn push(self) -> Self::PushStream {
        let stream = self.signal.push();
        let interval = IntervalStream::new(interval(self.period.clone()));
        let period = self.period;

        SamplePush {
            stream,
            current: None,
            interval,
            period,
        }
    }

    fn push_timeout(self, dur: Duration) -> Self::PushTimeoutStream {
        let stream = self.signal.push_timeout(dur);
        let interval = IntervalStream::new(interval(self.period.clone()));
        let period = self.period;

        SamplePushTimeout {
            stream,
            current: None,
            interval,
            period,
        }
    }
}

/// # Sample signal's pull stream.
///
/// Created by the method [SampleSignal::pull].
pub struct SamplePull<U, Pull>
where
    Pull: PullStream<Item = U>,
    U: Clone,
{
    pull_stream: Pull,
}
impl<U, Pull> PullStream for SamplePull<U, Pull>
where
    Pull: PullStream<Item = U>,
    U: Clone,
{
    type Item = Option<U>;

    fn pick(&mut self) -> Self::Item {
        Some(self.pull_stream.pick())
    }
}

/// # Sample signal's push stream.
///
/// Created by the method [SampleSignal::push].
#[pin_project(project = SamplePushProj)]
#[stream(push, item = U)]
pub struct SamplePush<U, Push>
where
    Push: PushStream<Item = U>,
    U: Clone,
{
    #[pin]
    stream: Push,
    current: Option<U>,
    #[pin]
    interval: IntervalStream,
    period: Duration,
}
impl<U, Push> PushStream for SamplePush<U, Push>
where
    Push: PushStream<Item = U>,
    U: Clone,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();

        // updating current value
        match project.stream.poll_update(cx) {
            Poll::Ready(value) => {
                *project.current = Some(value);
            }
            Poll::Pending => (),
        };

        match project.interval.poll_next(cx) {
            Poll::Ready(Some(_)) => Poll::Ready(
                project
                    .current
                    .as_ref()
                    .expect("there should be a value")
                    .clone(),
            ),
            Poll::Ready(None) => panic!("should not end"),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// # Sample signal's push timeout stream.
///
/// Created by the method [SampleSignal::push_timeout].
#[pin_project(project = SamplePushTimeoutProj)]
#[stream(timeout, item = Option<U>)]
pub struct SamplePushTimeout<U, PushT>
where
    PushT: PushTimeoutStream<Item = U>,
    U: Clone,
{
    #[pin]
    stream: PushT,
    current: Option<U>,
    #[pin]
    interval: IntervalStream,
    period: Duration,
}
impl<U, PushT> PushTimeoutStream for SamplePushTimeout<U, PushT>
where
    PushT: PushTimeoutStream<Item = U>,
    U: Clone,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();

        // updating current value
        match project.stream.poll_timeout(cx) {
            Poll::Ready(value) => {
                *project.current = Some(value);
            }
            Poll::Pending => (),
        };

        match project.interval.poll_next(cx) {
            Poll::Ready(Some(_)) => Poll::Ready(project.current.clone()),
            Poll::Ready(None) => panic!("should not end"),
            Poll::Pending => Poll::Pending,
        }
    }
}
