//! Shareable stream.

use futures::{
    task::{self, ArcWake},
    Stream,
};
use slab::Slab;
use std::{
    cell::UnsafeCell,
    pin::Pin,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering::{Acquire, SeqCst},
        },
        Arc, Mutex,
    },
    task::{Context, Poll, Waker},
};

const IDLE: usize = 0;
const POLLING: usize = 1;
const VALUE: usize = 2;
const POISONED: usize = 3;

const NULL_KEY: usize = usize::max_value();

impl<T: ?Sized> StreamShared for T where T: Stream {}

/// `StreamShared` trait.
pub trait StreamShared: Stream {
    fn shared(self) -> Shared<Self>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        Shared::new(self)
    }
}

/// Stream for the `shared` method in the trait `StreamShared`.
pub struct Shared<S: Stream> {
    inner: Arc<Inner<S>>,
    waker_key: usize,
    updated_key: usize,
}

impl<S: Stream> Shared<S> {
    pub fn new(stream: S) -> Self {
        let notifier = Arc::new(Notifier {
            state: AtomicUsize::new(IDLE),
            wakers: Mutex::new(Slab::new()),
            updated: Mutex::new(Slab::new()),
        });
        let waker = task::waker(notifier.clone());

        let inner = Inner {
            value: UnsafeCell::new(None),
            stream: UnsafeCell::new(stream),
            notifier,
            waker,
        };

        let mut wakers_guard = inner.notifier.wakers.lock().unwrap();
        let waker_key = wakers_guard.insert(None);
        drop(wakers_guard);

        let mut updated_guard = inner.notifier.updated.lock().unwrap();
        let updated_key = updated_guard.insert(true);
        drop(updated_guard);

        Self {
            inner: Arc::new(inner),
            waker_key,
            updated_key,
        }
    }

    /// Safety: callers must first ensure that `inner.state`
    /// is `VALUE`
    unsafe fn take_output(&mut self) -> Option<S::Item>
    where
        <S as Stream>::Item: Clone,
    {
        assert_eq!(self.inner.notifier.state.load(Acquire), VALUE);
        let result = self.inner.clone().clone_output();
        self.inner.update(&mut self.updated_key, false);
        result
    }
}

impl<S> Stream for Shared<S>
where
    S: Stream,
    S::Item: Clone,
{
    type Item = S::Item;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> std::task::Poll<std::option::Option<<S as Stream>::Item>> {
        let this = &mut *self;
        let inner = this.inner.clone();

        inner.record_waker(&mut this.waker_key, cx);
        inner.record_updated(&mut this.updated_key);

        // Fast path for when the wrapped stream has already completed
        if inner.notifier.state.load(Acquire) == VALUE && inner.is_updated(&mut this.updated_key) {
            // Safety: We're in the VALUE state
            return unsafe { Poll::Ready(this.take_output()) };
        }

        loop {
            match inner
                .notifier
                .state
                .compare_exchange(IDLE, POLLING, SeqCst, SeqCst)
                .unwrap_or_else(|x| x)
            {
                IDLE => {
                    // Lock acquired, fall through
                    inner.poll_signal(cx);
                    break;
                }
                POLLING => {
                    // Another task is currently polling
                    break;
                }
                VALUE => {
                    // Value hasn't changed
                    let all_see = inner
                        .notifier
                        .updated
                        .lock()
                        .unwrap()
                        .iter()
                        .all(|(_, updated)| !*updated);
                    if all_see {
                        inner.notifier.state.store(IDLE, SeqCst)
                    } else {
                        break;
                    }
                }
                POISONED => panic!("inner stream panicked during poll"),
                _ => unreachable!(),
            }
        }

        Poll::Pending
    }
}

impl<S> Clone for Shared<S>
where
    S: Stream,
{
    fn clone(&self) -> Self {
        let mut wakers_guard = self.inner.notifier.wakers.lock().unwrap();
        let waker_key = wakers_guard.insert(None);

        let mut updated_guard = self.inner.notifier.updated.lock().unwrap();
        let updated_key = updated_guard.insert(true);

        Self {
            inner: self.inner.clone(),
            waker_key,
            updated_key,
        }
    }
}

struct Inner<S: Stream> {
    value: UnsafeCell<Option<S::Item>>,
    stream: UnsafeCell<S>,
    notifier: Arc<Notifier>,
    waker: Waker,
}

unsafe impl<S> Send for Inner<S>
where
    S: Stream + Send,
    S::Item: Send + Sync,
{
}

unsafe impl<S> Sync for Inner<S>
where
    S: Stream + Send,
    S::Item: Send + Sync,
{
}

impl<S> Inner<S>
where
    S: Stream,
{
    /// Safety: callers must first ensure that `self.inner.state`
    /// is `VALUE`
    unsafe fn output(&self) -> &Option<S::Item> {
        assert_eq!(self.notifier.state.load(Acquire), VALUE);
        &*self.value.get()
    }

    unsafe fn store_value(&self, value: Option<<S as Stream>::Item>) {
        unsafe {
            *self.value.get() = value;
        }
        self.notifier.state.store(VALUE, SeqCst);
        self.notifier
            .updated
            .lock()
            .unwrap()
            .iter_mut()
            .for_each(|(_, updated)| *updated = true)
    }
}

impl<S> Inner<S>
where
    S: Stream,
    S::Item: Clone,
{
    /// Registers the current task to receive a wakeup when we are awoken.
    fn record_waker(&self, waker_key: &mut usize, cx: &mut Context<'_>) {
        let mut wakers_guard = self.notifier.wakers.lock().unwrap();

        let new_waker = cx.waker();

        if *waker_key == NULL_KEY {
            *waker_key = wakers_guard.insert(Some(new_waker.clone()));
        } else {
            match wakers_guard[*waker_key] {
                Some(ref old_waker) if new_waker.will_wake(old_waker) => {}
                // Could use clone_from here, but Waker doesn't specialize it.
                ref mut slot => *slot = Some(new_waker.clone()),
            }
        }
        debug_assert!(*waker_key != NULL_KEY);
    }

    /// Registers the current task to know if they received the value.
    fn record_updated(&self, updated_key: &mut usize) {
        let mut updated_guard = self.notifier.updated.lock().unwrap();

        if *updated_key == NULL_KEY {
            *updated_key = updated_guard.insert(true);
        }
        debug_assert!(*updated_key != NULL_KEY);
    }

    /// Change the boolean saying the value is updated.
    fn update(&self, updated_key: &mut usize, updated: bool) {
        let mut updated_guard = self.notifier.updated.lock().unwrap();

        if *updated_key == NULL_KEY {
            self.record_updated(updated_key);
        }

        updated_guard[*updated_key] = updated;
    }

    /// Get the boolean saying the value is updated.
    fn is_updated(&self, updated_key: &mut usize) -> bool {
        let updated_guard = self.notifier.updated.lock().unwrap();

        if *updated_key == NULL_KEY {
            self.record_updated(updated_key);
        }

        updated_guard[*updated_key]
    }

    /// Safety: callers must first ensure that `inner.state`
    /// is `VALUE`
    unsafe fn clone_output(self: Arc<Self>) -> Option<S::Item> {
        assert_eq!(self.notifier.state.load(Acquire), VALUE);
        match Arc::try_unwrap(self) {
            Ok(inner) => inner.value.into_inner(),
            Err(inner) => inner.output().clone(),
        }
    }

    fn poll_signal(&self, cx: &mut Context<'_>) {
        let stream = unsafe { Pin::new_unchecked(&mut *self.stream.get()) };

        match stream.poll_next(cx) {
            Poll::Ready(value) => {
                self.waker.wake_by_ref();
                unsafe { self.store_value(value) };
            }
            Poll::Pending => {}
        }
    }
}

struct Notifier {
    state: AtomicUsize,
    wakers: Mutex<Slab<Option<Waker>>>,
    updated: Mutex<Slab<bool>>,
}

impl ArcWake for Notifier {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let wakers = &mut *arc_self.wakers.lock().unwrap();
        for (_, opt_waker) in wakers {
            if let Some(waker) = opt_waker.take() {
                waker.wake();
            }
        }
    }
}

#[cfg(test)]
mod shared {
    use crate::shared::StreamShared;
    use futures::{stream, StreamExt};
    use futures_signals::signal::{Broadcaster, Mutable, SignalExt};

    #[tokio::test]
    async fn should_share_stream_between_threads() {
        let stream = stream::iter(1..5)
            .map(|value| {
                println!("input: {value}");
                value
            })
            .shared();
        let shared1 = stream.clone();
        let shared2 = stream;
        let _ = tokio::join!(
            tokio::spawn(
                shared1
                    .map(|value| value * 2)
                    .for_each(|value| async move { println!("shared1: {value}") }),
            ),
            tokio::spawn(
                shared2
                    .map(|value| value + 1)
                    .for_each(|value| async move { println!("shared2: {value}") }),
            ),
        );
    }

    #[tokio::test]
    async fn should_share_signals_between_threads() {
        let mutable = Mutable::new(1);
        let broadcaster = Broadcaster::new(mutable.signal());
        let signal1 = broadcaster.signal();
        let signal2 = broadcaster.signal();

        let _ = tokio::join!(
            tokio::spawn(
                signal1
                    .map(|value| value * 2)
                    .for_each(|value| async move { println!("signal1: {value}") }),
            ),
            tokio::spawn(
                signal2
                    .map(|value| value + 1)
                    .for_each(|value| async move { println!("signal2: {value}") }),
            ),
            tokio::spawn(stream::iter(1..5).for_each(move |value| {
                let mutable = mutable.clone();
                println!("input: {value}");
                async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    mutable.set(value)
                }
            })),
        );
    }
}
