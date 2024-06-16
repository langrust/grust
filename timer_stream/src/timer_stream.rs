use crate::{Timer, TimerQueue, Timing};
use futures::{Future, Stream};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::time::{sleep_until, Instant, Sleep};

/// # Combine two streams into a priority queue.
#[pin_project(project = TimerStreamProj)]
pub struct TimerStream<S, T, const N: usize>
where
    S: Stream<Item = (T, Instant)>,
{
    #[pin]
    stream: S,
    end: bool,
    queue: TimerQueue<T, N>,
    #[pin]
    sleep: Sleep,
    sleeping_timer: Option<(T, Instant)>,
}
impl<S, T, const N: usize> Stream for TimerStream<S, T, N>
where
    S: Stream<Item = (T, Instant)>,
    T: Timing + PartialEq,
{
    type Item = Option<S::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut project = self.project();
        let queue = project.queue;

        let mut abort = false;
        if !*project.end {
            loop {
                // println!("take timers");
                // collect arriving timers
                match project.stream.as_mut().poll_next(cx) {
                    // the stream have a value
                    Poll::Ready(Some((kind, pushed_instant))) => {
                        // if it is sleeping timer's kind then abort sleep
                        if let Some((sleeping_timer, _)) = project.sleeping_timer.as_ref() {
                            if kind.reset() && kind.eq(sleeping_timer) {
                                abort = true;
                            }
                        }
                        let deadline = pushed_instant + kind.get_duration();
                        queue.push(Timer::from_deadline(deadline.into(), kind));
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

        if abort || project.sleep.as_mut().poll(cx).is_ready() {
            match queue.pop() {
                Some(timer) => {
                    let (timer_kind, timer_deadline) = timer.get_kind_and_deadline();
                    project.sleep.reset(timer_deadline.into());
                    let output = std::mem::replace(
                        project.sleeping_timer,
                        Some((timer_kind, timer_deadline.into())),
                    );
                    Poll::Ready(Some(output))
                }
                None => {
                    if project.sleeping_timer.is_some() {
                        let output = std::mem::take(project.sleeping_timer);
                        Poll::Ready(Some(output))
                    } else {
                        if *project.end {
                            Poll::Ready(None)
                        } else {
                            Poll::Pending
                        }
                    }
                }
            }
        } else {
            Poll::Pending
        }
    }
}
pub fn timer_stream<S, T, const N: usize>(stream: S) -> TimerStream<S, T, N>
where
    S: Stream<Item = (T, Instant)>,
    T: Timing,
{
    TimerStream {
        stream,
        end: false,
        queue: TimerQueue::new(),
        sleep: sleep_until(Instant::now()),
        sleeping_timer: None,
    }
}

#[cfg(test)]
mod timer_stream {
    use std::{collections::HashMap, sync::Arc, time::Duration};

    use crate::{timer_stream::timer_stream, Timing};
    use futures::{SinkExt, StreamExt};
    use rand::distributions::{Distribution, Uniform};
    use tokio::{
        join,
        sync::RwLock,
        time::{sleep, Instant},
    };
    use ServiceTimers::*;

    #[derive(Debug, PartialEq)]
    enum ServiceTimers {
        Period10ms(usize),
        Period15ms(usize),
        Timeout20ms(usize),
        Timeout30ms(usize),
    }
    impl Timing for ServiceTimers {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                Period10ms(_) => std::time::Duration::from_millis(100),
                Period15ms(_) => std::time::Duration::from_millis(150),
                Timeout20ms(_) => std::time::Duration::from_millis(200),
                Timeout30ms(_) => std::time::Duration::from_millis(300),
            }
        }

        fn reset(&self) -> bool {
            match self {
                Period10ms(_) | Period15ms(_) => false,
                Timeout20ms(_) | Timeout30ms(_) => true,
            }
        }
    }
    impl ServiceTimers {
        fn set_id(&mut self, id: usize) {
            match self {
                Period10ms(old_id) | Period15ms(old_id) | Timeout20ms(old_id)
                | Timeout30ms(old_id) => *old_id = id,
            }
        }
        fn get_id(&self) -> usize {
            match self {
                Period10ms(old_id) | Period15ms(old_id) | Timeout20ms(old_id)
                | Timeout30ms(old_id) => *old_id,
            }
        }
    }

    #[tokio::test]
    async fn should_give_timers_in_order() {
        let stream = futures::stream::iter(vec![
            (Period15ms(0), Instant::now()),
            (Timeout30ms(1), Instant::now()),
            (Timeout20ms(2), Instant::now()),
            (Period10ms(3), Instant::now()),
        ]);
        let timers = timer_stream::<_, _, 10>(stream);
        tokio::pin!(timers);

        let mut v = vec![];
        while let Some(value) = timers.next().await {
            if let Some((value, _)) = value {
                v.push(value)
            }
        }
        assert_eq!(
            v,
            vec![Period10ms(3), Period15ms(0), Timeout20ms(2), Timeout30ms(1)]
        )
    }

    #[tokio::test]
    async fn should_give_elements_in_order_with_delay() {
        let stream = futures::stream::iter(vec![
            (Period15ms(0), Instant::now()),
            (Timeout30ms(1), Instant::now()),
            (Timeout20ms(2), Instant::now()),
            (Period10ms(3), Instant::now()),
        ])
        .chain(
            futures::stream::iter(vec![
                (Timeout20ms(4), Instant::now()),
                (Period10ms(5), Instant::now()),
                (Timeout30ms(6), Instant::now()),
                (Period15ms(7), Instant::now()),
            ])
            .then(|e| async move {
                sleep(Duration::from_millis(5)).await;
                e
            }),
        );
        tokio::pin!(stream);

        let timers = timer_stream::<_, _, 10>(stream);
        tokio::pin!(timers);

        let mut v = vec![];
        while let Some(value) = timers.next().await {
            v.push(value);
            sleep(Duration::from_millis(1)).await;
        }
        // println!("{v:?}")
    }

    struct TimerInfos {
        pushed_instant: Instant,
    }
    impl TimerInfos {
        fn new(pushed_instant: Instant) -> Self {
            TimerInfos { pushed_instant }
        }
    }
    struct TimersManager {
        timer_sink: futures::channel::mpsc::Sender<(ServiceTimers, Instant)>,
        timers: Arc<RwLock<HashMap<usize, TimerInfos>>>,
        fresh_id: usize,
    }
    impl TimersManager {
        fn new(
            timer_sink: futures::channel::mpsc::Sender<(ServiceTimers, Instant)>,
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
            let pushed_instant = Instant::now();
            let timer_infos = TimerInfos::new(pushed_instant);

            self.timer_sink.send((timer, pushed_instant)).await.unwrap();
            self.timers.write().await.insert(self.fresh_id, timer_infos);

            self.fresh_id += 1;
        }
    }

    #[tokio::test]
    async fn timers_deadlines_should_be_respected() {
        let (timer_sink, stream) = futures::channel::mpsc::channel::<(ServiceTimers, Instant)>(100);
        let timers: Arc<RwLock<HashMap<usize, TimerInfos>>> =
            Arc::new(RwLock::new(Default::default()));

        let handler_2 = tokio::spawn({
            let timers = timers.clone();
            async move {
                let timer_stream = timer_stream::<_, _, 100>(stream);
                tokio::pin!(timer_stream);
                while let Some(value) = timer_stream.next().await {
                    if let Some((kind, deadline)) = value {
                        let timer_id = kind.get_id();
                        let pushed_instant =
                            timers.read().await.get(&timer_id).unwrap().pushed_instant;

                        // asserting that deadlines are respected
                        let timer_duration = deadline.duration_since(pushed_instant);
                        let elapsed = Instant::now().duration_since(pushed_instant);
                        println!("elapsed: {:?} deadline: {:?}", elapsed, timer_duration);
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
