use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio_stream::Stream;

#[derive(Debug)]
pub enum Union<T1, T2> {
    E1(T1),
    E2(T2),
}

pub fn prio_queue<S1, S2>(stream_1: S1, stream_2: S2) -> PrioQueue<S1, S2>
where
    S1: Stream,
    S2: Stream,
{
    PrioQueue {
        stream_1,
        end_1: false,
        stream_2,
        end_2: false,
        queue: vec![],
    }
}

/// # Combine two streams into a priority queue.
#[pin_project(project = PrioQueueProj)]
pub struct PrioQueue<S1, S2>
where
    S1: Stream,
    S2: Stream,
{
    #[pin]
    stream_1: S1,
    end_1: bool,
    #[pin]
    stream_2: S2,
    end_2: bool,
    queue: Vec<Union<<S1 as Stream>::Item, <S2 as Stream>::Item>>,
}
impl<S1, S2> Stream for PrioQueue<S1, S2>
where
    S1: Stream,
    S2: Stream,
{
    type Item = Union<S1::Item, S2::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut project = self.project();
        let queue = project.queue;

        // collect arriving values
        if !*project.end_1 || !*project.end_2 {
            loop {
                let queuing = if !*project.end_1 {
                    match project.stream_1.as_mut().poll_next(cx) {
                        // the first stream have a value
                        Poll::Ready(Some(value)) => {
                            queue.push(Union::E1(value));
                            true
                        }
                        // the first stream is waiting
                        Poll::Pending => false,
                        // the first stream ended
                        Poll::Ready(None) => {
                            *project.end_1 = false;
                            false
                        }
                    }
                } else {
                    false
                };
                if !*project.end_2 {
                    match project.stream_2.as_mut().poll_next(cx) {
                        // the second stream have a value
                        Poll::Ready(Some(value)) => queue.push(Union::E2(value)),
                        // both streams are waiting
                        Poll::Pending => {
                            if !queuing {
                                break;
                            }
                        }
                        // the second stream ended
                        Poll::Ready(None) => {
                            *project.end_2 = false;
                            if !queuing {
                                break;
                            }
                        }
                    }
                }
            }
        }

        if queue.is_empty() {
            if *project.end_1 && *project.end_2 {
                Poll::Ready(None)
            } else {
                Poll::Pending
            }
        } else {
            Poll::Ready(queue.pop())
        }
    }
}

#[cfg(test)]
mod test {
    use colored::Colorize;
    use std::time::Duration;
    use tokio::{
        join,
        sync::mpsc::channel,
        time::{sleep, Instant},
    };
    use tokio_stream::{wrappers::ReceiverStream, StreamExt};

    use crate::prio_queue;

    #[tokio::test]
    async fn main() -> Result<(), String> {
        let (tx_1, rx_1) = channel::<i64>(1);
        let (tx_2, rx_2) = channel::<&str>(1);
        let stream_1 = ReceiverStream::new(rx_1);
        let stream_2 = ReceiverStream::new(rx_2);
        let mut prio = prio_queue(stream_1, stream_2);

        let handler_1 = tokio::spawn(async move {
            let end = Instant::now() + Duration::from_millis(100);
            loop {
                if Instant::now() > end {
                    return Ok(());
                }
                println!("{}", format!("big sleep").green());
                sleep(Duration::from_millis(10)).await;
                if let Err(e) = tx_1.send(0).await {
                    return Err(format!("output receiver dropped ({e})"));
                }
            }
        });

        let handler_2 = tokio::spawn(async move {
            let end = Instant::now() + Duration::from_millis(100);
            loop {
                if Instant::now() > end {
                    return Ok(());
                }
                println!("{}", format!("small sleep").blue());
                sleep(Duration::from_millis(5)).await;
                if let Err(e) = tx_2.send("a").await {
                    return Err(format!("output receiver dropped ({e})"));
                }
                if let Err(e) = tx_2.send("a").await {
                    return Err(format!("output receiver dropped ({e})"));
                }
                if let Err(e) = tx_2.send("a").await {
                    return Err(format!("output receiver dropped ({e})"));
                }
            }
        });

        let handler_3 = tokio::spawn(async move {
            loop {
                println!("{}", format!("tiny sleep").yellow());
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
