use pin_project::pin_project;
use std::{
    cmp::Ordering,
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
    pub fn new(order: F) -> Self {
        PrioQueue {
            queue: array_init::array_init(|_| Default::default()),
            order,
            len: 0,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn push(&mut self, value: T) {
        todo!()
    }
    pub fn pop(&mut self) -> Option<T> {
        todo!()
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
    S::Item: Default,
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
    use colored::Colorize;
    use std::{cmp::Ordering, sync::Arc, time::Duration};
    use tokio::{
        join,
        sync::mpsc::channel,
        time::{sleep, Instant},
    };
    use tokio_stream::{wrappers::ReceiverStream, StreamExt};

    use crate::prio_stream;

    #[derive(Debug, Default)]
    pub enum Union<T1, T2> {
        #[default]
        E0,
        E1(T1),
        E2(T2),
    }
    impl<T1, T2> Union<T1, T2> {
        pub fn order(v1: &Self, v2: &Self) -> Ordering {
            todo!()
        }
    }

    #[tokio::test]
    async fn main() -> Result<(), String> {
        let (tx, rx) = channel::<Union<i64, &str>>(1);
        let sender = Arc::new(tx);
        let stream = ReceiverStream::new(rx);
        let mut prio = prio_stream::<_, _, 10>(stream, Union::order);

        let handler_1 = tokio::spawn({
            let sender = sender.clone();
            async move {
                let end = Instant::now() + Duration::from_millis(100);
                loop {
                    if Instant::now() > end {
                        return Ok(());
                    }
                    sleep(Duration::from_millis(10)).await;
                    println!("{}", format!("E1(0)").green());
                    if let Err(e) = sender.send(Union::E1(0)).await {
                        return Err(format!("output receiver dropped ({e})"));
                    }
                }
            }
        });

        let handler_2 = tokio::spawn(async move {
            let end = Instant::now() + Duration::from_millis(100);
            loop {
                if Instant::now() > end {
                    return Ok(());
                }
                sleep(Duration::from_millis(5)).await;
                println!("{}", format!("E2(\"a\")").blue());
                if let Err(e) = sender.send(Union::E2("a")).await {
                    return Err(format!("output receiver dropped ({e})"));
                }
                println!("{}", format!("E2(\"a\")").blue());
                if let Err(e) = sender.send(Union::E2("a")).await {
                    return Err(format!("output receiver dropped ({e})"));
                }
                println!("{}", format!("E2(\"a\")").blue());
                if let Err(e) = sender.send(Union::E2("a")).await {
                    return Err(format!("output receiver dropped ({e})"));
                }
            }
        });

        let handler_3 = tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1)).await;
                if let Some(x) = prio.next().await {
                    println!("{}", format!("{x:?}").red());
                } else {
                    return;
                }
            }
        });

        let (res_1, res_2, res_3) = join!(handler_1, handler_2, handler_3);
        res_1.unwrap()?;
        res_2.unwrap()?;
        res_3.unwrap();
        Ok(())
    }
}
