use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_signals::signal::Signal;
use pin_project::pin_project;

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

