use futures::Stream;
use pin_project::pin_project;
use shared_stream::{Share, Shared};
use std::{
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use stream_proc_macros::stream;

use crate::stream::{pull_stream::PullStream, push_stream::PushStream};
use crate::{signal::Signal, stream::push_timeout_stream::PushTimeoutStream};

/// Creates a [SharedSignal] that produce a shareable signal from the given signal.
pub fn shared<T, S>(signal: S) -> SharedSignal<T, S>
where
    S: Signal<T>,
    T: Clone,
{
    SharedSignal {
        inner: Arc::new(Mutex::new(Inner::Signal(signal))),
        phantom: PhantomData,
    }
}

enum Inner<T, S>
where
    S: Signal<T>,
    T: Clone,
{
    Default(PhantomData<T>),
    Signal(S),
    Pull(Arc<Mutex<S::PullStream>>),
    Push(Shared<S::PushStream>),
    PushTimeout(Shared<S::PushTimeoutStream>),
}
impl<T, S> Default for Inner<T, S>
where
    S: Signal<T>,
    T: Clone,
{
    fn default() -> Self {
        Inner::Default(PhantomData)
    }
}

/// # Shared signal.
///
/// A signal sharedping the function to every value from the source signal.
/// It implements Signal trait, allowing to sample it synchronously
/// (creating a pull stream [SharedPull]) or asynchronously
/// (creating a push stream [SharedPush]).
pub struct SharedSignal<T, S>
where
    S: Signal<T>,
    T: Clone,
{
    inner: Arc<Mutex<Inner<T, S>>>,
    phantom: PhantomData<T>,
}
impl<T, S> Signal<T> for SharedSignal<T, S>
where
    S: Signal<T>,
    T: Clone,
{
    type PullStream = SharedPull<T, S::PullStream>;
    type PushStream = SharedPush<T, S::PushStream>;
    type PushTimeoutStream = SharedPushTimeout<T, S::PushTimeoutStream>;

    /// /!\ deprecated
    fn pull(self) -> Self::PullStream {
        let mut guard = self.inner.as_ref().lock().unwrap();
        match std::mem::take(&mut *guard) {
            // if no pull stream have been created
            Inner::Signal(signal) => {
                // create pull stream
                let pull_stream = Arc::new(Mutex::new(signal.pull()));
                // update inner by storing pull stream
                let _ = std::mem::replace(&mut *guard, Inner::Pull(pull_stream.clone()));
                // return pull stream
                SharedPull {
                    pull_stream: pull_stream.clone(),
                }
            }
            // if pull stream have been created
            Inner::Pull(pull_stream) => {
                // return pull stream
                SharedPull {
                    pull_stream: pull_stream.clone(),
                }
            }
            // otherwise, error
            _ => panic!("another type of stream have been created"),
        }
    }

    fn push(self) -> Self::PushStream {
        let mut guard = self.inner.as_ref().lock().unwrap();
        match std::mem::take(&mut *guard) {
            // if no push stream have been created
            Inner::Signal(signal) => {
                // create push stream
                let push_stream = signal.push().shared();
                // update inner by storing push stream
                let _ = std::mem::replace(&mut *guard, Inner::Push(push_stream.clone()));
                // return push stream
                SharedPush {
                    stream: push_stream.clone(),
                }
            }
            // if push stream have been created
            Inner::Push(push_stream) => {
                // return push stream
                SharedPush {
                    stream: push_stream.clone(),
                }
            }
            // otherwise, error
            _ => panic!("another type of stream have been created"),
        }
    }

    fn push_timeout(self, dur: std::time::Duration) -> Self::PushTimeoutStream {
        let mut guard = self.inner.as_ref().lock().unwrap();
        match std::mem::take(&mut *guard) {
            // if no push_timeout stream have been created
            Inner::Signal(signal) => {
                // create push_timeout stream
                let push_timeout_stream = signal.push_timeout(dur).shared();
                // update inner by storing push_timeout stream
                let _ =
                    std::mem::replace(&mut *guard, Inner::PushTimeout(push_timeout_stream.clone()));
                // return push_timeout stream
                SharedPushTimeout {
                    stream: push_timeout_stream.clone(),
                }
            }
            // if push_timeout stream have been created
            Inner::PushTimeout(push_timeout_stream) => {
                // return push_timeout stream
                SharedPushTimeout {
                    stream: push_timeout_stream.clone(),
                }
            }
            // otherwise, error
            _ => panic!("another type of stream have been created"),
        }
    }
}
impl<T, S> Clone for SharedSignal<T, S>
where
    S: Signal<T>,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

/// # Shared signal's pull stream.
///
/// Created by the method [SharedSignal::pull].
pub struct SharedPull<T, Pull>
where
    Pull: PullStream<Item = T>,
    T: Clone,
{
    pull_stream: Arc<Mutex<Pull>>,
}
impl<T, Pull> PullStream for SharedPull<T, Pull>
where
    Pull: PullStream<Item = T>,
    T: Clone,
{
    type Item = T;

    fn pick(&mut self) -> Self::Item {
        let mut guard = self.pull_stream.lock().unwrap();
        guard.pick()
    }
}

/// # Shared signal's push stream.
///
/// Created by the method [SharedSignal::push].
#[pin_project(project = SharedPushProj)]
#[stream(push, item = T)]
pub struct SharedPush<T, Push>
where
    Push: PushStream<Item = T>,
    T: Clone,
{
    #[pin]
    stream: Shared<Push>,
}
impl<T, Push> PushStream for SharedPush<T, Push>
where
    Push: PushStream<Item = T>,
    T: Clone,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_next(cx) {
            Poll::Ready(Some(value)) => Poll::Ready(value),
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => panic!("should not end"),
        }
    }
}

/// # Shared signal's push timeout stream.
///
/// Created by the method [SharedSignal::push_timeout].
#[pin_project(project = SharedPushTimeoutProj)]
#[stream(timeout, item = T)]
pub struct SharedPushTimeout<T, PushT>
where
    PushT: PushTimeoutStream<Item = T>,
    T: Clone,
{
    #[pin]
    stream: Shared<PushT>,
}
impl<T, PushT> PushTimeoutStream for SharedPushTimeout<T, PushT>
where
    PushT: PushTimeoutStream<Item = T>,
    T: Clone,
{
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_next(cx) {
            Poll::Ready(Some(value)) => Poll::Ready(value),
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => panic!("should not end"),
        }
    }
}
