use crate::{GetMillis, Timer, TimerQueue};
use futures::Stream;
use pin_project::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::time::{sleep_until, Instant, Sleep};

/// # Combine two streams into a priority queue.
#[pin_project(project = TimerStreamProj)]
pub struct TimerStream<S, const N: usize>
where
    S: Stream,
{
    #[pin]
    stream: S,
    end: bool,
    #[pin]
    sleep: Sleep,
    sleep_until: Instant,
    sleeping_timer: Option<S::Item>,
    queue: TimerQueue<S::Item, N>,
}
impl<S, const N: usize> Stream for TimerStream<S, N>
where
    S: Stream,
    S::Item: GetMillis + Default,
{
    type Item = Option<S::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut project = self.project();
        let queue = project.queue;

        if !*project.end {
            loop {
                println!("take timers");
                // collect arriving timers
                match project.stream.as_mut().poll_next(cx) {
                    // the stream have a value
                    Poll::Ready(Some(kind)) => {
                        let remaining = project.sleep_until.duration_since(Instant::now());
                        let deadline = kind.get_millis() - remaining;
                        println!(
                            "put timer of deadline {:?}, but remaining {deadline:?}",
                            kind.get_millis()
                        );
                        queue.push(Timer::from_duration(deadline, kind));
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

        match project.sleep.as_mut().poll(cx) {
            Poll::Ready(_) => match queue.pop() {
                Some(timer) => {
                    let (timer_kind, timer_deadline) = timer.get_kind_and_deadline();
                    let deadline = Instant::now() + timer_deadline;
                    println!("sleep for {timer_deadline:?}, i.e. until {deadline:?}");
                    *project.sleep_until = deadline;
                    project.sleep.reset(deadline);
                    let timer_kind = std::mem::replace(project.sleeping_timer, Some(timer_kind));
                    Poll::Ready(Some(timer_kind))
                }
                None => {
                    println!("do not sleep");
                    // queue is empty
                    if *project.end {
                        Poll::Ready(None)
                    } else {
                        Poll::Pending
                    }
                }
            },
            Poll::Pending => {
                println!("sleeping...");
                Poll::Pending
            }
        }
    }
}
pub fn timer_stream<S, const N: usize>(stream: S) -> TimerStream<S, N>
where
    S: Stream,
    S::Item: GetMillis + Default,
{
    TimerStream {
        stream,
        end: false,
        queue: TimerQueue::new(),
        sleep: sleep_until(Instant::now()),
        sleep_until: Instant::now(),
        sleeping_timer: None,
    }
}

#[cfg(test)]
mod timer_stream {
    use std::{collections::HashMap, sync::Arc, time::Duration};

    use crate::{timer_stream::timer_stream, GetMillis};
    use futures::{SinkExt, StreamExt};
    use rand::distributions::{Distribution, Uniform};
    use tokio::{
        join,
        sync::RwLock,
        time::{sleep, Instant},
    };
    use ServiceTimers::*;

    #[derive(Default, Debug, PartialEq)]
    enum ServiceTimers {
        #[default]
        NoTimer,
        Period10ms(usize),
        Period15ms(usize),
        Timeout20ms(usize),
        Timeout30ms(usize),
    }
    impl GetMillis for ServiceTimers {
        fn get_millis(&self) -> std::time::Duration {
            match self {
                NoTimer => panic!("no timer"),
                Period10ms(_) => std::time::Duration::from_millis(10),
                Period15ms(_) => std::time::Duration::from_millis(15),
                Timeout20ms(_) => std::time::Duration::from_millis(20),
                Timeout30ms(_) => std::time::Duration::from_millis(30),
            }
        }
    }
    impl ServiceTimers {
        fn set_id(&mut self, id: usize) {
            match self {
                NoTimer => panic!("no timer"),
                Period10ms(old_id) | Period15ms(old_id) | Timeout20ms(old_id)
                | Timeout30ms(old_id) => *old_id = id,
            }
        }
        fn get_id(&self) -> usize {
            match self {
                NoTimer => panic!("no timer"),
                Period10ms(old_id) | Period15ms(old_id) | Timeout20ms(old_id)
                | Timeout30ms(old_id) => *old_id,
            }
        }
    }

    #[tokio::test]
    async fn should_give_timers_in_order() {
        let stream = futures::stream::iter(vec![
            Period15ms(0),
            Timeout30ms(1),
            Timeout20ms(2),
            Period10ms(3),
        ]);
        let timers = timer_stream::<_, 10>(stream);
        tokio::pin!(timers);

        let mut v = vec![];
        while let Some(Some(value)) = timers.next().await {
            v.push(value)
        }
        assert_eq!(
            v,
            vec![Period10ms(3), Period15ms(0), Timeout20ms(2), Timeout30ms(1)]
        )
    }

    #[tokio::test]
    async fn should_give_elements_in_order_with_delay() {
        let stream = futures::stream::iter(vec![
            Period15ms(0),
            Timeout30ms(1),
            Timeout20ms(2),
            Period10ms(3),
        ])
        .chain(
            futures::stream::iter(vec![
                Timeout20ms(4),
                Period10ms(5),
                Timeout30ms(6),
                Period15ms(7),
            ])
            .then(|e| async move {
                sleep(Duration::from_millis(5)).await;
                e
            }),
        );
        tokio::pin!(stream);

        let timers = timer_stream::<_, 10>(stream);
        tokio::pin!(timers);

        let mut v = vec![];
        while let Some(value) = timers.next().await {
            v.push(value);
            sleep(Duration::from_millis(1)).await;
        }
        println!("{v:?}")
    }

    struct TimerInfos {
        duration: Duration,
        pushed_instant: Instant,
    }
    impl TimerInfos {
        fn new(duration: Duration, pushed_instant: Instant) -> Self {
            TimerInfos {
                duration,
                pushed_instant,
            }
        }
    }
    struct TimersManager {
        timer_sink: futures::channel::mpsc::Sender<ServiceTimers>,
        timers: Arc<RwLock<HashMap<usize, TimerInfos>>>,
        fresh_id: usize,
    }
    impl TimersManager {
        fn new(
            timer_sink: futures::channel::mpsc::Sender<ServiceTimers>,
            timers: Arc<RwLock<HashMap<usize, TimerInfos>>>,
        ) -> Self {
            TimersManager {
                timer_sink,
                timers,
                fresh_id: 0,
            }
        }
        async fn send_timer(&mut self, mut timer: ServiceTimers) {
            timer.set_id(self.fresh_id);
            let timer_infos = TimerInfos::new(timer.get_millis(), Instant::now());

            self.timer_sink.send(timer).await.unwrap();
            self.timers.write().await.insert(self.fresh_id, timer_infos);

            self.fresh_id += 1;
        }
    }

    #[tokio::test]
    async fn timers_deadlines_should_be_respected() {
        let (timer_sink, stream) = futures::channel::mpsc::channel::<ServiceTimers>(100);
        let timers: Arc<RwLock<HashMap<usize, TimerInfos>>> =
            Arc::new(RwLock::new(Default::default()));

        let handler_2 = tokio::spawn({
            let timers = timers.clone();
            async move {
                let timer_stream = timer_stream::<_, 100>(stream);
                tokio::pin!(timer_stream);
                while let Some(timer) = timer_stream.next().await {
                    if let Some(timer) = timer {
                        let timer_id = timer.get_id();
                        let pushed_instant =
                            timers.read().await.get(&timer_id).unwrap().pushed_instant;

                        // asserting that deadlines are respected
                        let timer_duration = timer.get_millis();
                        let elapsed = Instant::now().duration_since(pushed_instant);
                        println!(
                            "elapsed: {:?} ; timer: {:?}, pushed_instant : {:?}",
                            elapsed, timer_duration, pushed_instant
                        );
                    }
                }
            }
        });

        let handler_1 = tokio::spawn({
            let rng = rand::thread_rng();
            let distrib = Uniform::from(1..=6);
            let samples = distrib.sample_iter(rng).take(100).collect::<Vec<_>>();
            let mut sample_stream = futures::stream::iter(samples);
            async move {
                let mut timer_manager = TimersManager::new(timer_sink, timers);
                while let Some(sample) = sample_stream.next().await {
                    match sample {
                        0 => timer_manager.send_timer(Period10ms(0)).await,
                        1 => timer_manager.send_timer(Period15ms(0)).await,
                        2 => timer_manager.send_timer(Timeout20ms(0)).await,
                        3 => timer_manager.send_timer(Timeout30ms(0)).await,
                        _ => tokio::time::sleep(Duration::from_millis(10)).await,
                    }
                }
            }
        });

        let (_, _) = join!(handler_1, handler_2);
    }
}
