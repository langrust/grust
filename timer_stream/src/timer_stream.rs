use crate::{GetMillis, Timer, TimerQueue};
use pin_project::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::time::{sleep_until, Instant, Sleep};
use tokio_stream::Stream;

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
    queue: TimerQueue<S::Item, N>,
}
impl<S, const N: usize> Stream for TimerStream<S, N>
where
    S: Stream,
    S::Item: GetMillis + Default,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut project = self.project();
        let queue = project.queue;

        if !*project.end {
            loop {
                // collect arriving timers
                match project.stream.as_mut().poll_next(cx) {
                    // the stream have a value
                    Poll::Ready(Some(kind)) => {
                        queue.push(Timer::init(kind));
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
                    project.sleep.reset(deadline);
                    Poll::Ready(Some(timer_kind))
                }
                None => {
                    // queue is empty
                    if *project.end {
                        Poll::Ready(None)
                    } else {
                        Poll::Pending // todo: does sleep still exist?
                    }
                }
            },
            Poll::Pending => Poll::Pending,
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
    }
}

#[cfg(test)]
mod timer_stream {
    use std::{collections::HashMap, time::Duration};

    use crate::{timer_stream::timer_stream, GetMillis, Timer, TimerQueue};
    use rand::distributions::{Distribution, Uniform};
    use tokio::time::sleep;
    use tokio_stream::StreamExt;
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
        let stream = tokio_stream::iter(vec![
            Period15ms(0),
            Timeout30ms(1),
            Timeout20ms(2),
            Period10ms(3),
        ]);
        let timers = timer_stream::<_, 10>(stream);
        tokio::pin!(timers);

        let mut v = vec![];
        while let Some(value) = timers.next().await {
            v.push(value)
        }
        assert_eq!(
            v,
            vec![Period10ms(3), Period15ms(0), Timeout20ms(2), Timeout30ms(1)]
        )
    }
    struct TimerInfos {
        duration: Duration,
        pushed_time: Duration,
    }
    impl TimerInfos {
        fn new(duration: Duration, pushed_time: Duration) -> Self {
            TimerInfos {
                duration,
                pushed_time,
            }
        }
    }
    struct TimersManager {
        timer_stream: TimerQueue<ServiceTimers, 10>,
        timers: HashMap<usize, TimerInfos>,
        fresh_id: usize,
        global_time: Duration,
    }
    impl TimersManager {
        fn new() -> Self {
            let timer_stream = TimerQueue::<ServiceTimers, 10>::new();
            TimersManager {
                timer_stream,
                timers: Default::default(),
                fresh_id: 0,
                global_time: Duration::from_millis(0),
            }
        }
        fn insert_timer(&mut self, mut timer: ServiceTimers) {
            // if queue is full, do nothing (not the purpose of the test)
            if self.timer_stream.is_full() {
                return;
            }

            timer.set_id(self.fresh_id);
            let timer_infos = TimerInfos::new(timer.get_millis(), self.global_time);

            self.timer_stream.push(Timer::init(timer));
            self.timers.insert(self.fresh_id, timer_infos);

            self.fresh_id += 1;
        }
        fn pop_timer(&mut self) {
            // if queue is empty, do nothing (not the purpose of the test)
            if self.timer_stream.is_empty() {
                return;
            }
            let timer = self.timer_stream.pop().unwrap();
            let timer_id = timer.get_kind().get_id();
            let timer_infos = self.timers.get(&timer_id).unwrap();

            // asserting that deadlines are respected
            let global_time = self.global_time;
            let timer_popped_deadline = *timer.get_deadline();
            let timer_pushed_time = timer_infos.pushed_time;
            let timer_duration = timer_infos.duration;
            assert!(global_time + timer_popped_deadline == timer_pushed_time + timer_duration);

            // update global time
            self.global_time = global_time + timer_popped_deadline;
        }
    }

    #[tokio::test]
    async fn should_give_elements_in_order_with_delay() {
        let stream = tokio_stream::iter(vec![
            Period15ms(0),
            Timeout30ms(1),
            Timeout20ms(2),
            Period10ms(3),
        ])
        .chain(
            tokio_stream::iter(vec![
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

    #[test]
    fn timers_deadlines_should_be_respected() {
        let mut timer_manager = TimersManager::new();
        let mut rng = rand::thread_rng();
        let distrib = Uniform::from(1..=6);

        for _ in 0..100 {
            match distrib.sample(&mut rng) {
                0 => timer_manager.insert_timer(Period10ms(0)),
                1 => timer_manager.insert_timer(Period15ms(0)),
                2 => timer_manager.insert_timer(Timeout20ms(0)),
                3 => timer_manager.insert_timer(Timeout30ms(0)),
                _ => timer_manager.pop_timer(),
            }
        }
    }
}
