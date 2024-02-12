use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use pin_project::pin_project;

/// Constructs an event out of a stream and a callback.
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
/// # use futures_streams::stream::always;
/// # use interface_frp_streams_events::event::StreamEvent;
/// # let input = always(1);
/// let event = input.event(3, |x| async move {x * 10});
/// ```
pub trait StreamEvent: Stream {
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
impl<T: ?Sized> StreamEvent for T where T: Stream {}

#[pin_project(project = EventProj)]
pub struct Event<A, B, C>
where
    A: Stream,
{
    #[pin]
    inner_stream: Option<A>,
    #[pin]
    future: Option<B>,
    compute: bool,
    callback: C,
    buffer: Vec<A::Item>,
    bound: usize,
}

impl<A, B, C> Event<A, B, C>
where
    A: Stream,
    B: Future,
    C: FnMut(A::Item) -> B,
{
    /// Create a new `Event`
    pub fn new(stream: A, bound: usize, callback: C) -> Self {
        Self {
            inner_stream: Some(stream),
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

impl<A, B, C> Stream for Event<A, B, C>
where
    A: Stream,
    B: Future,
    C: FnMut(A::Item) -> B,
{
    type Item = B::Output;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let EventProj {
            mut inner_stream,
            mut future,
            compute,
            callback,
            buffer,
            bound,
        } = self.project();

        loop {
            match inner_stream
                .as_mut()
                .as_pin_mut()
                .map(|inner_stream| inner_stream.poll_next(cx))
            {
                // inner_stream is over
                None => {
                    // if the event does not compute then...
                    if !*compute {
                        match remove_first(buffer) {
                            // if buffer contains value then compute
                            Some(value) => {
                                let value = Some(callback(value));
                                future.set(value);
                                *compute = true;
                            }
                            // otherwise, nothing else to do
                            None => return Poll::Ready(None),
                        };
                    }
                }
                // if inner_stream output is None then inner_stream is over
                Some(Poll::Ready(None)) => {
                    inner_stream.set(None);
                    continue;
                }
                // if inner_stream output is Some value then...
                Some(Poll::Ready(Some(new_value))) => {
                    // if the event already computes then bufferize
                    if *compute {
                        if buffer.len() < *bound {
                            // but only if the buffer has no overflow
                            println!("\tpush value into buffer event");
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
                // if inner_stream is pending then...
                Some(Poll::Pending) => {
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
    use std::cell::Cell;
    use std::rc::Rc;
    use std::{future::poll_fn, task::Poll};

    use futures::StreamExt;
    use futures_signals::signal::{Mutable, SignalExt};

    use crate::{event::StreamEvent, util::with_noop_context};

    #[test]
    fn should_bufferize_when_already_computing() {
        let mutable = Rc::new(Mutable::new(1));

        let block = Rc::new(Cell::new(true));

        let mut event = {
            let block = block.clone();

            mutable.signal().to_stream().event(3, move |value| {
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

        with_noop_context(|cx| {
            let change = event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Pending);
            mutable.set(2);

            let change = event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Pending);
            block.set(false);
            mutable.set(3);

            let change = event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Ready(Some(1)));

            let change = event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Ready(Some(2)));

            let change = event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Ready(Some(3)));
        });
    }

    #[should_panic]
    #[test]
    fn should_panic_when_buffer_overflow() {
        let mutable = Rc::new(Mutable::new(1));

        let block = Rc::new(Cell::new(true));

        let mut event = {
            let block = block.clone();

            mutable.signal().to_stream().event(3, move |value| {
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

        with_noop_context(|cx| {
            let change = event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Pending);
            mutable.set(2);

            let change = event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Pending);
            mutable.set(3);

            let change = event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Pending);
            mutable.set(4);

            let change = event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Pending);
            mutable.set(5);

            let change = event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Pending);
        });
    }

    #[test]
    fn should_be_able_to_be_mapped_into_an_event() {
        let mutable = Rc::new(Mutable::new(1));

        let first_lock = Rc::new(Cell::new(true));

        let event = {
            let first_lock = first_lock.clone();

            mutable.signal().to_stream().event(3, move |value| {
                let first_lock = first_lock.clone();

                poll_fn(move |_| {
                    if first_lock.get() {
                        Poll::Pending
                    } else {
                        Poll::Ready(value)
                    }
                })
            })
        };

        let second_lock = Rc::new(Cell::new(true));

        let mut mapped_event = {
            let second_lock = second_lock.clone();

            event.event(3, move |value| {
                let second_lock = second_lock.clone();

                poll_fn(move |_| {
                    if second_lock.get() {
                        Poll::Pending
                    } else {
                        Poll::Ready(value)
                    }
                })
            })
        };

        with_noop_context(|cx| {
            let change = mapped_event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Pending);
            mutable.set(2);

            let change = mapped_event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Pending);
            first_lock.set(false);
            mutable.set(3);

            let change = mapped_event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Pending);
            second_lock.set(false);
            mutable.set(4);

            let change = mapped_event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Ready(Some(1)));

            let change = mapped_event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Ready(Some(2)));

            let change = mapped_event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Ready(Some(3)));

            let change = mapped_event.poll_next_unpin(cx);
            assert_eq!(change, Poll::Ready(Some(4)));
        });
    }
}
