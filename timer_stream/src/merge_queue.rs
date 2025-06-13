use crate::{Timer, Timing};
use std::{cmp::Ordering, fmt::Debug, time::Instant};

#[allow(clippy::wrong_self_convention)]
pub trait MergeTimer {
    type TimerTy;

    fn into(self) -> Option<Timer<Self::TimerTy>>;
    fn from(timer: Timer<Self::TimerTy>) -> Self;
    fn is_timer(&self) -> bool;
    fn into_ref(&self) -> Option<Timer<&Self::TimerTy>>;
    fn get_instant(&self) -> &Instant;
}

/// Merge queue.
///
/// It store timers in deadline order, along with timed values.
/// Timed values can be turned into timers and conversely.
pub struct MergeQueue<U, const N: usize>
where
    U: MergeTimer,
{
    queue: [Option<U>; N],
    len: usize,
}

impl<U, const N: usize> Default for MergeQueue<U, N>
where
    U: MergeTimer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<U, const N: usize> MergeQueue<U, N>
where
    U: MergeTimer,
{
    /// Create empty queue.
    pub fn new() -> Self {
        MergeQueue {
            queue: std::array::from_fn(|_| None),
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
    /// Pop the most urgent value from the queue.
    pub fn pop(&mut self) -> Option<U> {
        if self.is_empty() {
            None
        } else {
            let res = std::mem::take(&mut self.queue[self.len - 1]);
            self.len -= 1;
            res
        }
    }
    /// Push a value in queue.
    ///
    /// # Panics
    ///
    /// This function will panic if the queue is full.
    pub fn push_value(&mut self, value: U) {
        // safety: panics if pushed out of bound
        if self.is_full() {
            panic!("out of bound")
        }

        // puts the value at the right place
        for index in 0..self.len {
            let curr = self.queue[index].as_mut().unwrap();
            match value.get_instant().cmp(curr.get_instant()) {
                Ordering::Greater | Ordering::Equal => {
                    self.queue[index..=self.len].rotate_right(1);
                    self.queue[index] = Some(value);
                    self.len += 1;
                    return;
                }
                Ordering::Less => (),
            }
        }
        // if not inserted, then put it at the end
        self.queue[self.len] = Some(value);
        self.len += 1;
    }
    /// Push a timer in queue.
    ///
    /// # Panics
    ///
    /// This function will panic if the queue is full.
    pub fn push_timer(&mut self, timer: Timer<U::TimerTy>) {
        self.push_value(MergeTimer::from(timer))
    }
}
impl<U, const N: usize> MergeQueue<U, N>
where
    U: MergeTimer,
    U::TimerTy: Timing + PartialEq,
{
    /// Reset a timer in the queue.
    ///
    /// This will remove the previous version of the timer and add the new one.
    /// This will push the timer if not in the queue.
    ///
    /// # Panics
    ///
    /// This function will panic if the queue is full.
    /// This function will panic if the timer is not resettable.
    pub fn reset(&mut self, timer: Timer<U::TimerTy>) {
        if !timer.get_kind().do_reset() {
            panic!("not resettable")
        }

        // removes resetted timers
        for index in (0..self.len).rev() {
            let curr = self.queue[index].as_mut().unwrap();

            // if curr should be resetted then remove it
            // and add its dealine to the next timer (if it exists)
            if let Some(curr_timer) = curr.into_ref() {
                if *curr_timer.get_kind() == timer.get_kind() {
                    self.queue[index] = None;
                    self.queue[index..self.len].rotate_left(1);
                    self.len -= 1;
                }
            }
        }
        // pushes the timer at the right place
        self.push_timer(timer)
    }
}
impl<U, const N: usize> MergeQueue<U, N>
where
    U: MergeTimer + Debug,
{
    pub fn println(&self) {
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
impl<U, const N: usize> From<MergeQueue<U, N>> for Vec<U>
where
    U: MergeTimer,
{
    fn from(val: MergeQueue<U, N>) -> Self {
        let v = val
            .queue
            .into_iter()
            .take(val.len)
            .map(|timer| timer.unwrap())
            .collect::<Vec<_>>();
        debug_assert!(v.len() == val.len);
        v
    }
}

#[cfg(test)]
mod merge_queue {
    use std::time::{Duration, Instant};

    use crate::{MergeQueue, Timer, Timing};
    use rand::distributions::{Distribution, Uniform};
    use ServiceTimer::*;

    use super::MergeTimer;

    #[derive(Debug, PartialEq)]
    enum ServiceTimer {
        Period10ms,
        Period15ms,
        Timeout20ms,
        Timeout30ms,
    }
    impl Timing for ServiceTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                Period10ms => std::time::Duration::from_millis(10),
                Period15ms => std::time::Duration::from_millis(15),
                Timeout20ms => std::time::Duration::from_millis(20),
                Timeout30ms => std::time::Duration::from_millis(30),
            }
        }

        fn do_reset(&self) -> bool {
            match self {
                Period10ms | Period15ms => false,
                Timeout20ms | Timeout30ms => true,
            }
        }
    }

    #[derive(Debug, PartialEq)]
    enum Value {
        Speed(i64, Instant),
        Timer(ServiceTimer, Instant),
    }
    impl MergeTimer for Value {
        type TimerTy = ServiceTimer;
        fn into(self) -> Option<Timer<ServiceTimer>> {
            match self {
                Value::Speed(_, _) => None,
                Value::Timer(kind, deadline) => Some(Timer::from_deadline(deadline, kind)),
            }
        }

        fn from(timer: Timer<ServiceTimer>) -> Self {
            let (kind, deadline) = timer.get_kind_and_deadline();
            Value::Timer(kind, deadline)
        }

        fn is_timer(&self) -> bool {
            match self {
                Value::Speed(_, _) => false,
                Value::Timer(_, _) => true,
            }
        }

        fn into_ref(&self) -> Option<Timer<&ServiceTimer>> {
            match self {
                Value::Speed(_, _) => None,
                Value::Timer(kind, deadline) => Some(Timer::from_deadline(*deadline, kind)),
            }
        }

        fn get_instant(&self) -> &Instant {
            match self {
                Value::Speed(_, instant) | Value::Timer(_, instant) => instant,
            }
        }
    }

    #[test]
    fn new_should_create_empty_queue() {
        let merge_queue = MergeQueue::<Value, 10>::new();
        assert!(merge_queue.is_empty())
    }

    #[test]
    fn push_should_insert_timer_according_to_deadline() {
        let now = Instant::now();
        let mut merge_queue = MergeQueue::<Value, 10>::new();
        merge_queue.push_timer(Timer::init(Period15ms, now));
        merge_queue.push_timer(Timer::init(Timeout30ms, now));
        merge_queue.push_timer(Timer::init(Timeout20ms, now));
        merge_queue.push_value(Value::Speed(20, now));
        merge_queue.push_timer(Timer::init(Period10ms, now));
        let v: Vec<_> = merge_queue.into();
        assert_eq!(
            v,
            vec![
                Value::Timer(Timeout30ms, now + Duration::from_millis(30)),
                Value::Timer(Timeout20ms, now + Duration::from_millis(20)),
                Value::Timer(Period15ms, now + Duration::from_millis(15)),
                Value::Timer(Period10ms, now + Duration::from_millis(10)),
                Value::Speed(20, now),
            ]
        )
    }

    #[test]
    fn pop_should_remove_the_earliest_value() {
        let mut now = Instant::now();
        let mut merge_queue = MergeQueue::<Value, 10>::new();
        merge_queue.push_timer(Timer::init(Period15ms, now));
        merge_queue.push_timer(Timer::init(Timeout30ms, now));
        merge_queue.push_timer(Timer::init(Timeout20ms, now));
        merge_queue.push_value(Value::Speed(20, now));
        merge_queue.push_timer(Timer::init(Period10ms, now));
        merge_queue.println();
        assert!(merge_queue.len() == 5);

        assert_eq!(merge_queue.pop(), Some(Value::Speed(20, now)));

        assert_eq!(
            merge_queue.pop(),
            Some(Value::Timer(Period10ms, now + Duration::from_millis(10)))
        );
        now += Duration::from_millis(10);

        assert!(merge_queue.len() == 3);
        merge_queue.push_timer(Timer::init(Period10ms, now));
        assert!(merge_queue.len() == 4);

        assert_eq!(
            merge_queue.pop(),
            Some(Value::Timer(Period15ms, now + Duration::from_millis(5)))
        );
        now += Duration::from_millis(5);

        assert!(merge_queue.len() == 3);
        assert_eq!(
            merge_queue.pop(),
            Some(Value::Timer(Timeout20ms, now + Duration::from_millis(5)))
        );
        now += Duration::from_millis(5);

        assert!(merge_queue.len() == 2);
        assert_eq!(
            merge_queue.pop(),
            Some(Value::Timer(Period10ms, now + Duration::from_millis(0)))
        );
        now += Duration::from_millis(0);

        assert!(merge_queue.len() == 1);
        assert_eq!(
            merge_queue.pop(),
            Some(Value::Timer(Timeout30ms, now + Duration::from_millis(10)))
        );

        assert!(merge_queue.is_empty());
        assert_eq!(merge_queue.pop(), None);
        assert!(merge_queue.is_empty());
    }

    struct TimersManager {
        merge_queue: MergeQueue<Value, 10>,
        global_time: Instant,
    }
    impl TimersManager {
        fn new() -> Self {
            let merge_queue = MergeQueue::<Value, 10>::new();
            TimersManager {
                merge_queue,
                global_time: Instant::now(),
            }
        }
        fn insert_timer(&mut self, timer: ServiceTimer) {
            // if queue is full, do nothing (not the purpose of the test)
            if self.merge_queue.is_full() {
                return;
            }
            self.merge_queue
                .push_timer(Timer::init(timer, self.global_time));
        }
        fn insert_speed(&mut self, speed: i64) {
            // if queue is full, do nothing (not the purpose of the test)
            if self.merge_queue.is_full() {
                return;
            }
            self.merge_queue
                .push_value(Value::Speed(speed, self.global_time));
        }
        fn reset_timer(&mut self, timer: ServiceTimer) {
            // if queue is full, do nothing (not the purpose of the test)
            if self.merge_queue.is_full() {
                return;
            }
            self.merge_queue.reset(Timer::init(timer, self.global_time));
        }
        fn pop_value(&mut self) {
            // if queue is empty, do nothing (not the purpose of the test)
            if self.merge_queue.is_empty() {
                return;
            }
            let value = self.merge_queue.pop().unwrap();
            // assert popping is time monotonic
            assert!(&self.global_time <= value.get_instant());
            // update global time
            self.global_time = *value.get_instant();
        }
    }

    #[test]
    fn popping_from_insters_should_be_time_monotonic() {
        let mut timer_manager = TimersManager::new();
        let mut rng = rand::thread_rng();
        let distrib = Uniform::from(1..=8);

        for _ in 0..100 {
            match distrib.sample(&mut rng) {
                0 => timer_manager.insert_timer(Period10ms),
                1 => timer_manager.insert_timer(Period15ms),
                2 => timer_manager.insert_timer(Timeout20ms),
                3 => timer_manager.insert_timer(Timeout30ms),
                4 => timer_manager.insert_speed(20),
                _ => timer_manager.pop_value(),
            }
        }
    }

    #[test]
    fn reset_should_insert_timer_according_to_deadline() {
        let mut merge_queue = MergeQueue::<Value, 10>::new();
        let now = Instant::now();
        merge_queue.push_timer(Timer::init(Period15ms, now));
        merge_queue.push_timer(Timer::init(Timeout30ms, now + Duration::from_millis(10)));
        merge_queue.push_timer(Timer::init(Timeout20ms, now));
        merge_queue.push_timer(Timer::init(Period10ms, now));
        merge_queue.reset(Timer::init(Timeout20ms, now + Duration::from_millis(15)));
        merge_queue.reset(Timer::init(Timeout30ms, now));

        let v: Vec<_> = merge_queue.into();
        assert_eq!(
            v,
            vec![
                Value::Timer(Timeout20ms, now + Duration::from_millis(35)),
                Value::Timer(Timeout30ms, now + Duration::from_millis(30)),
                Value::Timer(Period15ms, now + Duration::from_millis(15)),
                Value::Timer(Period10ms, now + Duration::from_millis(10)),
            ]
        )
    }

    #[test]
    fn reset_should_insert_unique_timer() {
        let mut merge_queue = MergeQueue::<Value, 10>::new();
        let now = Instant::now();
        merge_queue.push_timer(Timer::init(Period15ms, now));
        merge_queue.push_timer(Timer::init(Timeout30ms, now));
        merge_queue.push_timer(Timer::init(Timeout20ms, now));
        merge_queue.reset(Timer::init(Timeout30ms, now));
        let v: Vec<_> = merge_queue.into();
        assert_eq!(
            v,
            vec![
                Value::Timer(Timeout30ms, now + Duration::from_millis(30)),
                Value::Timer(Timeout20ms, now + Duration::from_millis(20)),
                Value::Timer(Period15ms, now + Duration::from_millis(15)),
            ]
        )
    }

    #[test]
    fn popping_from_insters_and_reset_should_be_time_monotonic() {
        let mut timer_manager = TimersManager::new();
        let mut rng = rand::thread_rng();
        let distrib = Uniform::from(1..=10);

        for _ in 0..100 {
            match distrib.sample(&mut rng) {
                0 => timer_manager.insert_timer(Period10ms),
                1 => timer_manager.insert_timer(Period15ms),
                2 => timer_manager.insert_timer(Timeout20ms),
                3 => timer_manager.insert_timer(Timeout30ms),
                4 => timer_manager.insert_speed(20),
                5 => timer_manager.reset_timer(Timeout20ms),
                6 => timer_manager.reset_timer(Timeout30ms),
                _ => timer_manager.pop_value(),
            }
        }
    }
}
