use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_signals::signal::Signal;
use pin_project::pin_project;

/// Constructs an event out of a signal and a callback.
///
/// The callback is applied to every value of the event.
/// If the callback tacks to much time and a new value comes
/// before the end of computation, then the new value is stored
/// into a buffer.
///
/// The buffer is bounded, if the bound is exceeded the code panics.
/// 
/// The callback needs to be a future.
///
/// # Example
/// ```rust
/// # use futures_signals::signal::always;
/// # use interface_frp_signals_events::event::SignalEvent;
/// # let input = always(1);
/// let event = input.event(3, |x| async move {x * 10});
/// ```
pub trait SignalEvent: Signal {
    #[inline]
    fn event<A, B>(self, bound: usize, callback: B) -> Event<Self, A, B>
    where
        A: Future,
        B: FnMut(Self::Item) -> A,
        Self: Sized,
    {
        Event::new(self, bound, callback)
    }
}
impl<T: ?Sized> SignalEvent for T where T: Signal {}

#[pin_project(project = EventProj)]
pub struct Event<A, B, C>
where
    A: Signal,
{
    #[pin]
    inner_signal: A,
    #[pin]
    future: Option<B>,
    compute: bool,
    callback: C,
    buffer: Vec<A::Item>,
    bound: usize,
}

impl<A, B, C> Event<A, B, C>
where
    A: Signal,
    B: Future,
    C: FnMut(A::Item) -> B,
{
    /// Create a new `Event`
    pub fn new(signal: A, bound: usize, callback: C) -> Self {
        Self {
            inner_signal: signal,
            future: None,
            compute: false,
            callback: callback,
            buffer: vec![],
            bound,
        }
    }
}

fn remove_first<T>(vec: &mut Vec<T>) -> Option<T> {
    if vec.is_empty() {
        return None;
    }
    Some(vec.remove(0))
}

impl<A, B, C> Signal for Event<A, B, C>
where
    A: Signal,
    B: Future,
    C: FnMut(A::Item) -> B,
    A::Item: Debug,
{
    type Item = B::Output;

    #[inline]
    fn poll_change(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<B::Output>> {
        let EventProj {
            mut inner_signal,
            mut future,
            compute,
            callback,
            buffer,
            bound,
        } = self.project();

        loop {
            match inner_signal.as_mut().poll_change(cx) {
                // if inner_signal output is None then inner_signal is over
                Poll::Ready(None) => return Poll::Ready(None),
                // if inner_signal output is Some value then...
                Poll::Ready(Some(new_value)) => {
                    // if the event already computes then bufferize
                    if *compute {
                        if buffer.len() < *bound {
                            // but only if the buffer has no overflow
                            buffer.push(new_value);
                        } else {
                            panic!("Event: buffer overflow")
                        }
                    } else {
                        // else, look if their are buffered values
                        match remove_first(buffer) {
                            // if buffer contains value then compute
                            Some(value) => {
                                buffer.push(new_value);
                                let value = Some(callback(value));
                                future.set(value);
                                *compute = true;
                            }
                            // otherwise, compute with new value
                            None => {
                                let value = Some(callback(new_value));
                                future.set(value);
                                *compute = true;
                            }
                        }
                    };
                    continue;
                }
                // if inner_signal is pending then...
                Poll::Pending => {
                    // if the event does not compute then...
                    if !*compute {
                        match remove_first(buffer) {
                            // if buffer contains value then compute
                            Some(value) => {
                                let value = Some(callback(value));
                                future.set(value);
                                *compute = true;
                            }
                            // otherwise, pass
                            None => {}
                        };
                    }
                }
            }
            break;
        }

        match future.as_mut().as_pin_mut().map(|future| future.poll(cx)) {
            None => return Poll::Pending,
            Some(Poll::Ready(value)) => {
                future.set(None);
                *compute = false;
                return Poll::Ready(Some(value));
            }
            Some(Poll::Pending) => return Poll::Pending,
        }
    }
}

#[cfg(test)]
mod event {
    use futures_signals::signal::Mutable;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::{future::poll_fn, task::Poll};

    use crate::{event::SignalEvent, util};

    #[test]
    fn should_bufferize_when_already_computing() {
        let mutable = Rc::new(Mutable::new(1));

        let block = Rc::new(Cell::new(true));

        let event = {
            let block = block.clone();

            mutable.signal().event(3, move |value| {
                let block = block.clone();

                poll_fn(move |_| {
                    if block.get() {
                        Poll::Pending
                    } else {
                        Poll::Ready(value)
                    }
                })
            })
        };

        util::ForEachSignal::new(event)
            .next({
                let mutable = mutable.clone();
                move |_, change| {
                    assert_eq!(change, Poll::Pending);
                    mutable.set(2);
                }
            })
            .next({
                let mutable = mutable.clone();
                move |_, change| {
                    assert_eq!(change, Poll::Pending);
                    block.set(false);
                    mutable.set(3);
                }
            })
            .next(|_, change| {
                assert_eq!(change, Poll::Ready(Some(1)));
            })
            .next(|_, change| {
                assert_eq!(change, Poll::Ready(Some(2)));
            })
            .next(|_, change| {
                assert_eq!(change, Poll::Ready(Some(3)));
            })
            .run();
    }

    #[should_panic]
    #[test]
    fn should_panic_when_buffer_overflow() {
        let mutable = Rc::new(Mutable::new(1));

        let block = Rc::new(Cell::new(true));

        let s = {
            let block = block.clone();

            mutable.signal().event(3, move |value| {
                let block = block.clone();

                poll_fn(move |_| {
                    if block.get() {
                        Poll::Pending
                    } else {
                        Poll::Ready(value)
                    }
                })
            })
        };

        util::ForEachSignal::new(s)
            .next({
                let mutable = mutable.clone();
                move |_, change| {
                    assert_eq!(change, Poll::Pending);
                    mutable.set(2);
                }
            })
            .next({
                let mutable = mutable.clone();
                move |_, change| {
                    assert_eq!(change, Poll::Pending);
                    mutable.set(3);
                }
            })
            .next({
                let mutable = mutable.clone();
                move |_, change| {
                    assert_eq!(change, Poll::Pending);
                    mutable.set(4);
                }
            })
            .next({
                let mutable = mutable.clone();
                move |_, change| {
                    assert_eq!(change, Poll::Pending);
                    mutable.set(5);
                }
            })
            .next({
                move |_, change| {
                    assert_eq!(change, Poll::Pending);
                }
            })
            .run();
    }
}
