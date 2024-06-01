use pin_project::pin_project;
use std::{
    cmp::Ordering,
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};
use tokio_stream::Stream;

pub struct PrioQueue<T, F, const N: usize>
where
    F: FnMut(&T, &T) -> Ordering,
{
    queue: [T; N],
    order: F,
    len: usize,
}
impl<T, F, const N: usize> PrioQueue<T, F, N>
where
    T: Default,
    F: FnMut(&T, &T) -> Ordering,
{
    /// Create empty queue.
    pub fn new(order: F) -> Self {
        PrioQueue {
            queue: array_init::array_init(|_| Default::default()),
            order,
            len: 0,
        }
    }
    /// Give the length of the queue.
    pub fn len(&self) -> usize {
        self.len
    }
    /// Tell if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Tell if the queue is full.
    pub fn is_full(&self) -> bool {
        self.len == N
    }
    /// Push a value in ordered queue.
    ///
    /// # Panics
    ///
    /// This function will panic if the queue is full.
    pub fn push(&mut self, value: T) {
        // safety: panics if pushed out of bound
        if self.is_full() {
            panic!("out of bound")
        }

        // puts the value at the right place
        for index in 0..self.len {
            match (self.order)(&value, &self.queue[index]) {
                Ordering::Greater | Ordering::Equal => {
                    self.queue[index..=self.len].rotate_right(1);
                    self.queue[index] = value;
                    self.len += 1;
                    return;
                }
                Ordering::Less => (),
            }
        }
        // if not inserted, then put it at the end
        self.queue[self.len] = value;
        self.len += 1;
    }
    /// Pop the smallest element of the queue.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let res = std::mem::take(&mut self.queue[self.len - 1]);
            self.len -= 1;
            Some(res)
        }
    }
    pub fn println(&self)
    where
        T: Debug,
    {
        if self.is_empty() {
            println!("[]")
        } else {
            print!("[");
            self.queue
                .iter()
                .take(self.len - 1)
                .for_each(|t| print!("{t:?}, "));
            println!("{:?}]", self.queue[self.len - 1])
        }
    }
}
impl<T, F, const N: usize> Into<Vec<T>> for PrioQueue<T, F, N>
where
    T: Default,
    F: FnMut(&T, &T) -> Ordering,
{
    fn into(self) -> Vec<T> {
        let v = self.queue.into_iter().take(self.len).collect::<Vec<_>>();
        debug_assert!(v.len() == self.len);
        v
    }
}

#[cfg(test)]
mod prio_queue {
    use std::cmp::Ordering;

    use crate::PrioQueue;

    fn order(a: &i32, b: &i32) -> Ordering {
        if a < b {
            Ordering::Less
        } else if a == b {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }

    #[test]
    fn new_should_create_empty_queue() {
        let prio_queue = PrioQueue::<_, _, 10>::new(order);
        assert!(prio_queue.is_empty())
    }

    #[test]
    fn push_should_insert_elements_according_to_order() {
        let mut prio_queue = PrioQueue::<_, _, 10>::new(order);
        prio_queue.push(3);
        prio_queue.push(4);
        prio_queue.push(-1);
        prio_queue.push(2);
        prio_queue.push(5);
        let v: Vec<_> = prio_queue.into();
        assert_eq!(v, vec![5, 4, 3, 2, -1])
    }

    #[test]
    fn pop_should_remove_the_smallest_element() {
        let mut prio_queue = PrioQueue::<_, _, 10>::new(order);
        prio_queue.push(3);
        prio_queue.push(4);
        prio_queue.push(2);
        prio_queue.push(5);
        assert!(prio_queue.len() == 4);
        assert_eq!(prio_queue.pop(), Some(2));
        assert!(prio_queue.len() == 3);
        prio_queue.push(-1);
        assert!(prio_queue.len() == 4);
        assert_eq!(prio_queue.pop(), Some(-1));
        assert!(prio_queue.len() == 3);
        assert_eq!(prio_queue.pop(), Some(3));
        assert!(prio_queue.len() == 2);
        assert_eq!(prio_queue.pop(), Some(4));
        assert!(prio_queue.len() == 1);
        assert_eq!(prio_queue.pop(), Some(5));
        assert!(prio_queue.len() == 0);
        assert_eq!(prio_queue.pop(), None);
        assert!(prio_queue.len() == 0);
    }
}

/// # Combine two streams into a priority queue.
#[pin_project(project = PrioStreamProj)]
pub struct PrioStream<S, F, const N: usize>
where
    S: Stream,
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
    S::Item: Default + Debug,
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
                    // the first stream have a value
                    Poll::Ready(Some(value)) => {
                        queue.push(value);
                    }
                    // the first stream is waiting
                    Poll::Pending => break,
                    // the first stream ended
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
    S::Item: Default,
    F: FnMut(&S::Item, &S::Item) -> Ordering,
{
    PrioStream {
        stream,
        end: false,
        queue: PrioQueue::new(order),
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;
    use tokio_stream::StreamExt;

    use crate::prio_stream;

    #[derive(Debug, Default, PartialEq)]
    pub enum Union<T1, T2> {
        #[default]
        E0,
        E1(T1),
        E2(T2),
    }
    impl<T1, T2> Union<T1, T2> {
        pub fn order(v1: &Self, v2: &Self) -> Ordering {
            match (v1, v2) {
                (Union::E0, Union::E0)
                | (Union::E1(_), Union::E1(_))
                | (Union::E2(_), Union::E2(_)) => Ordering::Equal,
                (Union::E0, _) | (Union::E1(_), Union::E2(_)) => Ordering::Less,
                (_, Union::E0) | (Union::E2(_), Union::E1(_)) => Ordering::Greater,
            }
        }
    }

    #[tokio::test]
    async fn main() {
        let stream = tokio_stream::iter(vec![
            Union::E1(0),
            Union::E2("a"),
            Union::E1(0),
            Union::E2("a"),
            Union::E1(0),
            Union::E2("a"),
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
            ]
        )
    }
}
