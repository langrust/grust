use futures::Stream;
use pin_project::pin_project;
use std::{
    cmp::Ordering,
    pin::Pin,
    task::{Context, Poll},
};

use crate::priority_stream::PrioQueue;

pub trait Reset {
    fn do_reset(&self) -> bool;
}

/// # Combine two streams into a priority queue.
#[pin_project(project = PrioStreamProj)]
pub struct PrioStream<S, F, const N: usize>
where
    S: Stream,
    S::Item: Reset + PartialEq,
    F: FnMut(&S::Item, &S::Item) -> Ordering,
{
    #[pin]
    stream: S,
    end: bool,
    queue: PrioQueue<S::Item, F, N>,
}
impl<S, F, const N: usize> Stream for PrioStream<S, F, N>
where
    S: Stream,
    S::Item: Reset + PartialEq,
    F: FnMut(&S::Item, &S::Item) -> Ordering,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut project = self.project();
        let queue = project.queue;

        if !*project.end {
            loop {
                // collect arriving values
                match project.stream.as_mut().poll_next(cx) {
                    // the stream have a value
                    Poll::Ready(Some(value)) => {
                        if value.do_reset() {
                            queue.reset(value);
                        } else {
                            queue.push(value);
                        }
                    }
                    // the stream is waiting
                    Poll::Pending => break,
                    // the stream ended
                    Poll::Ready(None) => {
                        *project.end = true;
                        break;
                    }
                }
            }
        }

        if queue.is_empty() {
            if *project.end {
                Poll::Ready(None)
            } else {
                Poll::Pending
            }
        } else {
            Poll::Ready(queue.pop())
        }
    }
}

pub fn prio_stream<S, F, const N: usize>(stream: S, order: F) -> PrioStream<S, F, N>
where
    S: Stream,
    S::Item: Reset + PartialEq,
    F: FnMut(&S::Item, &S::Item) -> Ordering,
{
    PrioStream {
        stream,
        end: false,
        queue: PrioQueue::new(order),
    }
}

#[cfg(test)]
mod prio_stream {
    use futures::StreamExt;
    use std::{cmp::Ordering, time::Duration};
    use tokio::time::sleep;

    use crate::priority_stream::prio_stream;

    use super::Reset;

    #[derive(Debug, PartialEq)]
    pub enum Union<T1, T2, T3> {
        E1(T1),
        E2(T2),
        E3(T3),
    }
    impl<T1, T2, T3> Union<T1, T2, T3> {
        pub fn order(v1: &Self, v2: &Self) -> Ordering {
            match (v1, v2) {
                (Union::E1(_), Union::E1(_))
                | (Union::E2(_), Union::E2(_))
                | (Union::E3(_), Union::E3(_)) => Ordering::Equal,
                (Union::E3(_), _) => Ordering::Greater,
                (Union::E2(_), _) => Ordering::Greater,
                (Union::E1(_), _) => Ordering::Less,
            }
        }
    }
    impl<T1, T2, T3> Reset for Union<T1, T2, T3> {
        fn do_reset(&self) -> bool {
            match self {
                Union::E1(_) => false,
                Union::E2(_) => false,
                Union::E3(_) => true,
            }
        }
    }

    #[tokio::test]
    async fn should_give_elements_in_order() {
        let stream = futures::stream::iter(vec![
            Union::E1(0),
            Union::E2("a"),
            Union::E1(0),
            Union::E3(1.0),
            Union::E2("a"),
            Union::E1(0),
            Union::E3(1.0),
            Union::E2("a"),
            Union::E3(1.0),
        ]);
        let mut prio = prio_stream::<_, _, 10>(stream, Union::order);
        let mut v = vec![];
        while let Some(value) = prio.next().await {
            v.push(value)
        }
        assert_eq!(
            v,
            vec![
                Union::E1(0),
                Union::E1(0),
                Union::E1(0),
                Union::E2("a"),
                Union::E2("a"),
                Union::E2("a"),
                Union::E3(1.0),
            ]
        )
    }

    #[tokio::test]
    async fn should_give_elements_in_order_with_delay() {
        let stream = futures::stream::iter(vec![
            Union::E1(0),
            Union::E2("a"),
            Union::E1(0),
            Union::E3(1.0),
            Union::E2("a"),
            Union::E1(0),
            Union::E3(1.0),
            Union::E2("a"),
            Union::E3(1.0),
        ])
        .chain(
            futures::stream::iter(vec![
                Union::E1(0),
                Union::E2("a"),
                Union::E1(0),
                Union::E3(1.0),
                Union::E2("a"),
                Union::E1(0),
                Union::E3(1.0),
                Union::E2("a"),
                Union::E3(1.0),
            ])
            .then(|e| async move {
                sleep(Duration::from_millis(5)).await;
                e
            }),
        );
        tokio::pin!(stream);
        let mut prio = prio_stream::<_, _, 10>(stream, Union::order);
        let mut v = vec![];
        while let Some(value) = prio.next().await {
            v.push(value);
            sleep(Duration::from_millis(1)).await;
        }
        println!("{v:?}")
    }
}
