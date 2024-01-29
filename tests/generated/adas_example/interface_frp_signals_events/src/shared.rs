//! Shareable stream.

use futures::{
    task::{waker_ref, ArcWake},
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
        let inner = Inner {
            value: UnsafeCell::new(None),
            stream: UnsafeCell::new(stream),
            notifier: Arc::new(Notifier {
                state: AtomicUsize::new(IDLE),
                wakers: Mutex::new(Slab::new()),
                updated: Mutex::new(Slab::new()),
            }),
        };

        Self {
            inner: Arc::new(inner),
            waker_key: NULL_KEY,
            updated_key: NULL_KEY,
        }
    }

    /// Safety: callers must first ensure that `inner.state`
    /// is `VALUE`
    unsafe fn take_output(&mut self) -> Option<S::Item>
    where
        <S as Stream>::Item: Clone,
    {
        assert_eq!(self.inner.notifier.state.load(Acquire), VALUE);
        let result = self.inner.clone().take_or_clone_output();
        self.inner.update(&mut self.updated_key, false);
        self.inner.notifier.state.store(IDLE, SeqCst);
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

        // Fast path for when the wrapped stream has already completed
        if inner.notifier.state.load(Acquire) == VALUE && inner.is_updated(&mut this.updated_key) {
            // Safety: We're in the VALUE state
            return unsafe { Poll::Ready(this.take_output()) };
        }

        inner.record_waker(&mut this.waker_key, cx);
        inner.record_updated(&mut this.updated_key);

        match inner
            .notifier
            .state
            .compare_exchange(IDLE, POLLING, SeqCst, SeqCst)
            .unwrap_or_else(|x| x)
        {
            IDLE => {
                // Lock acquired, fall through
            }
            POLLING => {
                // Another task is currently polling, at this point we just want
                // to ensure that the waker for this task is registered
                return Poll::Pending;
            }
            VALUE => {
                // Safety: We're in the VALUE state
                return unsafe { Poll::Ready(this.take_output()) };
            }
            POISONED => panic!("inner stream panicked during poll"),
            _ => unreachable!(),
        }

        println!("yo");
        let waker = waker_ref(&inner.notifier);
        let mut cx = Context::from_waker(&waker);

        struct Reset<'a> {
            state: &'a AtomicUsize,
            did_not_panic: bool,
        }

        impl Drop for Reset<'_> {
            fn drop(&mut self) {
                if !self.did_not_panic {
                    self.state.store(POISONED, SeqCst);
                }
            }
        }

        let mut reset = Reset {
            state: &inner.notifier.state,
            did_not_panic: false,
        };

        let output = {
            let stream = unsafe { Pin::new_unchecked(&mut *inner.stream.get()) };

            let poll_result = stream.poll_next(&mut cx);
            reset.did_not_panic = true;

            match poll_result {
                Poll::Pending => {
                    if inner
                        .notifier
                        .state
                        .compare_exchange(POLLING, IDLE, SeqCst, SeqCst)
                        .is_ok()
                    {
                        // Success
                        drop(reset);
                        return Poll::Pending;
                    } else {
                        unreachable!()
                    }
                }
                Poll::Ready(output) => {
                    inner.update(&mut this.updated_key, true);
                    output
                }
            }
        };

        unsafe { inner.store_value(output) };

        // Wake all tasks
        let wakers_guard = inner.notifier.wakers.lock().unwrap();
        for (_, waker) in wakers_guard.iter() {
            if let Some(waker) = waker {
                waker.wake_by_ref();
            }
        }

        // Safety: We're in the VALUE state
        unsafe { Poll::Ready(this.take_output()) }
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
        let updated_key = updated_guard.insert(None);

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
            *updated_key = updated_guard.insert(Some(false));
        }
        debug_assert!(*updated_key != NULL_KEY);
    }

    /// Change the boolean saying the value is updated.
    fn update(&self, updated_key: &mut usize, updated: bool) {
        let mut updated_guard = self.notifier.updated.lock().unwrap();

        if *updated_key == NULL_KEY {
            self.record_updated(updated_key);
        }

        updated_guard[*updated_key] = Some(updated);
    }

    /// Get the boolean saying the value is updated.
    fn is_updated(&self, updated_key: &mut usize) -> bool {
        let updated_guard = self.notifier.updated.lock().unwrap();

        if *updated_key == NULL_KEY {
            self.record_updated(updated_key);
        }

        updated_guard[*updated_key].unwrap()
    }

    /// Safety: callers must first ensure that `inner.state`
    /// is `VALUE`
    unsafe fn take_or_clone_output(self: Arc<Self>) -> Option<S::Item> {
        assert_eq!(self.notifier.state.load(Acquire), VALUE);
        match Arc::try_unwrap(self) {
            Ok(inner) => inner.value.into_inner(),
            Err(inner) => inner.output().clone(),
        }
    }
}

struct Notifier {
    state: AtomicUsize,
    wakers: Mutex<Slab<Option<Waker>>>,
    updated: Mutex<Slab<Option<bool>>>,
}

impl ArcWake for Notifier {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let wakers = &mut *arc_self.wakers.lock().unwrap();
        for (waker_key, opt_waker) in wakers {
            println!("record: {waker_key}");
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

    #[tokio::test]
    async fn should_share_stream_between_threads() {
        let stream = stream::iter(1..5).shared();
        let shared1 = stream.clone();
        let shared2 = stream.clone();
        let _ = tokio::join!(
            tokio::spawn(
                shared1
                    .map(|value| value * 2)
                    .for_each(|value| async move { println!("{value}") }),
            ),
            tokio::spawn(
                shared2
                    .map(|value| value + 1)
                    .for_each(|value| async move { println!("{value}") }),
            ),
        );
    }
}
