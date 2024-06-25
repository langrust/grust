use crate::{MergeQueue, MergeTimer, Timer, Timing};
use futures::Stream;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

/// # Combine stream and timer into a priority queue.
#[pin_project(project = MergeTimerProj)]
pub struct MergeStream<Sv, St, const N: usize>
where
    Sv: Stream,
    Sv::Item: MergeTimer,
    St: Stream<Item = Timer<<Sv::Item as MergeTimer>::TimerTy>>,
{
    #[pin]
    stream: Sv,
    end_stream: bool,
    #[pin]
    timer: St,
    end_timer: bool,
    queue: MergeQueue<Sv::Item, N>,
}
impl<Sv, St, const N: usize> Stream for MergeStream<Sv, St, N>
where
    Sv: Stream,
    Sv::Item: MergeTimer,
    St: Stream<Item = Timer<<Sv::Item as MergeTimer>::TimerTy>>,
    <Sv::Item as MergeTimer>::TimerTy: Timing + PartialEq,
{
    type Item = Sv::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut project = self.project();
        let queue = project.queue;

        if !*project.end_timer {
            loop {
                // collect arriving timers
                match project.timer.as_mut().poll_next(cx) {
                    // the stream have a timer
                    Poll::Ready(Some(timer)) => {
                        if timer.get_kind().do_reset() {
                            queue.reset(timer)
                        } else {
                            queue.push_timer(timer)
                        }
                    }
                    // the stream is waiting
                    Poll::Pending => break,
                    // the stream ended
                    Poll::Ready(None) => {
                        *project.end_timer = true;
                        break;
                    }
                }
            }
        }

        if !*project.end_stream {
            loop {
                // collect arriving values
                match project.stream.as_mut().poll_next(cx) {
                    // the stream have a value
                    Poll::Ready(Some(value)) => queue.push_value(value),
                    // the stream is waiting
                    Poll::Pending => break,
                    // the stream ended
                    Poll::Ready(None) => {
                        *project.end_timer = true;
                        break;
                    }
                }
            }
        }

        if queue.is_empty() {
            if *project.end_stream && *project.end_timer {
                Poll::Ready(None)
            } else {
                Poll::Pending
            }
        } else {
            Poll::Ready(queue.pop())
        }
    }
}

/// Create a stream that can merge timers into another stream.
pub fn merge_stream<Sv, St, const N: usize>(stream: Sv, timer: St) -> MergeStream<Sv, St, N>
where
    Sv: Stream,
    Sv::Item: MergeTimer,
    St: Stream<Item = Timer<<Sv::Item as MergeTimer>::TimerTy>>,
{
    MergeStream {
        stream,
        end_stream: false,
        timer,
        end_timer: false,
        queue: MergeQueue::new(),
    }
}
