use crate::{Timer, TimerQueue, Timing};
use futures::{Future, Stream};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};
use tokio::time::{sleep_until, Sleep};

/// # Timer stream.
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
    sleep_infos: Option<(T, Instant)>,
}
impl<S, T, const N: usize> Stream for TimerStream<S, T, N>
where
    S: Stream<Item = (T, Instant)>,
    T: Timing + PartialEq,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut project = self.project();
        let queue = project.queue;

        let mut abort = false;
        if !*project.end {
            loop {
                // collect arriving timers
                match project.stream.as_mut().poll_next(cx) {
                    // the stream have a value
                    Poll::Ready(Some((kind, pushed_instant))) => {
                        // check if need to abort `sleep`
                        if let Some((sleeping_timer, sleeping_deadline)) =
                            project.sleep_infos.as_ref()
                        {
                            // if it is sleeping timer's kind then plan to abort `sleep`
                            if kind.do_reset() && kind.eq(sleeping_timer) {
                                abort = true;
                            } else 
                            // if urgent then plan to abort `sleep` and push current timer back into queue
                            if sleeping_deadline > &(pushed_instant + kind.get_duration()) {
                                abort = true;
                                let (sleeping_timer, sleeping_deadline) =
                                    project.sleep_infos.take().unwrap();
                                queue.push(Timer::from_deadline(sleeping_deadline, sleeping_timer))
                            }
                        }

                        // push new timer into queue
                        if kind.do_reset() {
                            queue.reset(Timer::init(kind, pushed_instant));
                        } else {
                            queue.push(Timer::init(kind, pushed_instant));
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

        // need to reset `sleep` (= `abort` planned or `sleep` terminated)
        if abort || project.sleep.as_mut().poll(cx).is_ready() {
            match queue.pop() {
                Some(timer) => {
                    let (timer_kind, timer_deadline) = timer.get_kind_and_deadline();
                    // reset `sleep` with popped timer
                    project.sleep.reset(timer_deadline.into());
                    // update `sleep_infos`
                    let output = std::mem::replace(
                        project.sleep_infos,
                        Some((timer_kind, timer_deadline.into())),
                    );
                    // if `sleep` terminated then send its infos
                    if !abort && output.is_some() {
                        Poll::Ready(output)
                    } else {
                        Poll::Pending
                    }
                }
                None => {
                    if project.sleep_infos.is_some() {
                        // if `sleep` terminated then send its infos
                        let output = project.sleep_infos.take();
                        Poll::Ready(output)
                    } else {
                        // if no more timers then end the stream
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
        sleep: sleep_until(tokio::time::Instant::now()),
        sleep_infos: None,
    }
}

#[cfg(test)]
mod timer_stream {
    use std::{
        collections::HashMap,
        sync::Arc,
        time::{Duration, Instant},
    };

    use crate::{timer_stream::timer_stream, Timing};
    use futures::{SinkExt, StreamExt};
    use rand::distributions::{Distribution, Uniform};
    use tokio::{
        join,
        sync::RwLock,
        time::{sleep, sleep_until},
    };
    use ServiceTimers::*;

    #[derive(Debug)]
    enum ServiceTimers {
        Period10ms(usize),
        Period15ms(usize),
        Timeout20ms(usize),
        Timeout30ms(usize),
    }
    impl PartialEq for ServiceTimers {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::Period10ms(_), Self::Period10ms(_))
                | (Self::Period15ms(_), Self::Period15ms(_))
                | (Self::Timeout20ms(_), Self::Timeout20ms(_))
                | (Self::Timeout30ms(_), Self::Timeout30ms(_)) => true,
                _ => false,
            }
        }
    }
    impl Timing for ServiceTimers {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                Period10ms(_) => std::time::Duration::from_millis(10),
                Period15ms(_) => std::time::Duration::from_millis(15),
                Timeout20ms(_) => std::time::Duration::from_millis(20),
                Timeout30ms(_) => std::time::Duration::from_millis(30),
            }
        }

        fn do_reset(&self) -> bool {
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
        let now = Instant::now();
        let stream = futures::stream::iter(vec![
            (Period15ms(0), now),
            (Timeout30ms(1), now),
            (Timeout20ms(2), now),
            (Period10ms(3), now),
        ]);
        let timers = timer_stream::<_, _, 10>(stream);
        tokio::pin!(timers);

        let mut v = vec![];
        while let Some((value, _)) = timers.next().await {
            v.push(value)
        }
        assert_eq!(
            v,
            vec![Period10ms(3), Period15ms(0), Timeout20ms(2), Timeout30ms(1)]
        )
    }

    #[tokio::test]
    async fn should_give_elements_in_order_with_delay() {
        let now = Instant::now();
        let next = now + Duration::from_millis(5);
        let stream = futures::stream::iter(vec![
            (Period15ms(0), now),
            (Timeout30ms(1), now),
            (Timeout20ms(2), now),
            (Period10ms(3), now),
        ])
        .chain(
            futures::stream::iter(vec![
                (Timeout20ms(4), next),
                (Period10ms(5), next),
                (Timeout30ms(6), next),
                (Period15ms(7), next),
            ])
            .then(|e| async move {
                sleep_until(next.into()).await;
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
        let control = vec![
            (
                ServiceTimers::Period10ms(3),
                now + ServiceTimers::Period10ms(3).get_duration(),
            ),
            (
                ServiceTimers::Period15ms(0),
                now + ServiceTimers::Period15ms(0).get_duration(),
            ),
            (
                ServiceTimers::Period10ms(5),
                next + ServiceTimers::Period10ms(5).get_duration(),
            ),
            (
                ServiceTimers::Period15ms(7),
                next + ServiceTimers::Period15ms(7).get_duration(),
            ),
            (
                ServiceTimers::Timeout20ms(4),
                next + ServiceTimers::Timeout20ms(4).get_duration(),
            ),
            (
                ServiceTimers::Timeout30ms(6),
                next + ServiceTimers::Timeout30ms(6).get_duration(),
            ),
        ];
        assert_eq!(v, control);
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
                while let Some((kind, deadline)) = timer_stream.next().await {
                    let timer_id = kind.get_id();
                    let pushed_instant = timers.read().await.get(&timer_id).unwrap().pushed_instant;

                    // asserting that deadlines are respected
                    let timer_duration = deadline.duration_since(pushed_instant);
                    let elapsed = Instant::now().duration_since(pushed_instant);
                    println!("elapsed: {:?} deadline: {:?}", elapsed, timer_duration);
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
