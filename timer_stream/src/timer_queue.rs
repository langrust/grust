use std::{cmp::Ordering, fmt::Debug, time::Duration};

/// A trait that gives duration from something (a timer kind, for example).
pub trait GetMillis {
    fn get_millis(&self) -> Duration;
}

/// Timer.
///
/// A timer has a `kind`, which is its identifier (period_component_c, timeout_event_e, etc).
/// It also has a deadline, to which it should tick.
#[derive(Debug, PartialEq)]
pub struct Timer<T> {
    deadline: Duration,
    kind: T,
}
impl<T> Timer<T>
where
    T: GetMillis,
{
    /// Initiate a new timer.
    pub fn init(kind: T) -> Timer<T> {
        Timer {
            deadline: kind.get_millis(),
            kind,
        }
    }
}
impl<T> Timer<T> {
    /// Get timer's kind.
    pub fn get_kind(&self) -> &T {
        &self.kind
    }
    /// Get timer's deadline.
    pub fn get_deadline(&self) -> &Duration {
        &self.deadline
    }
    /// Get timer's kind and deadline.
    pub fn get_kind_and_deadline(self) -> (T, Duration) {
        (self.kind, self.deadline)
    }
    /// Create a timer.
    #[cfg(test)]
    pub fn from_millis(millis: u64, kind: T) -> Self {
        Timer {
            deadline: std::time::Duration::from_millis(millis),
            kind,
        }
    }
}

/// Timer queue.
///
/// It store timers in deadline order.
/// Forall timer in the queue, the initial deadline is equal to the sum
/// of deadlines of the previous timers in the queue.
pub struct TimerQueue<T, const N: usize> {
    queue: [Option<Timer<T>>; N],
    len: usize,
}
impl<T, const N: usize> TimerQueue<T, N> {
    /// Create empty queue.
    pub fn new() -> Self {
        TimerQueue {
            queue: array_init::array_init(|_| None),
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
    /// Pop the most urgent timer from the queue.
    pub fn pop(&mut self) -> Option<Timer<T>> {
        if self.is_empty() {
            None
        } else {
            let res = std::mem::take(&mut self.queue[self.len - 1]);
            self.len -= 1;
            res
        }
    }
    /// Push a value in timer queue.
    ///
    /// # Panics
    ///
    /// This function will panic if the queue is full.
    pub fn push(&mut self, mut value: Timer<T>) {
        // safety: panics if pushed out of bound
        if self.is_full() {
            panic!("out of bound")
        }

        // puts the value at the right place
        for index in (0..self.len).rev() {
            let curr = self.queue[index].as_mut().unwrap();
            match value.deadline.cmp(&curr.deadline) {
                Ordering::Less => {
                    curr.deadline -= value.deadline;
                    self.queue[(index + 1)..=self.len].rotate_right(1);
                    self.queue[index + 1] = Some(value);
                    self.len += 1;
                    return;
                }
                Ordering::Equal => value.deadline = Duration::from_millis(0),
                Ordering::Greater => value.deadline -= curr.deadline,
            }
        }
        // if not inserted, then put it at the begining
        self.queue[0..=self.len].rotate_right(1);
        self.queue[0] = Some(value);
        self.len += 1;
    }
}
impl<T, const N: usize> TimerQueue<T, N>
where
    T: PartialEq,
{
    /// Reset a timer in the queue.
    ///
    /// This will remove the previous version of the timer and add the new one.
    /// This will push the timer if not in the queue.
    ///
    /// # Panics
    ///
    /// This function will panic if the queue is full.
    pub fn reset(&mut self, value: Timer<T>) {
        // removes resetted timers
        for index in (0..self.len).rev() {
            let curr = self.queue[index].as_mut().unwrap();

            // if curr should be resetted then remove it
            // and add its dealine to the next timer (if it exists)
            if &curr.kind == &value.kind {
                let old_timer = std::mem::take(&mut self.queue[index]).unwrap();
                self.queue[index..=self.len].rotate_left(1);
                self.len -= 1;
                if index > 0 {
                    let next_timer = self.queue[index - 1].as_mut().unwrap();
                    next_timer.deadline += old_timer.deadline;
                }
            }
        }
        // pushes the value at the right place
        self.push(value)
    }
}
impl<T, const N: usize> TimerQueue<T, N>
where
    T: Debug,
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
impl<T, const N: usize> Into<Vec<T>> for TimerQueue<T, N> {
    fn into(self) -> Vec<T> {
        let v = self
            .queue
            .into_iter()
            .take(self.len)
            .map(|timer| timer.unwrap().kind)
            .collect::<Vec<_>>();
        debug_assert!(v.len() == self.len);
        v
    }
}

#[cfg(test)]
mod timer_queue {
    use std::{collections::HashMap, time::Duration};

    use crate::{GetMillis, Timer, TimerQueue};
    use rand::distributions::{Distribution, Uniform};
    use ServiceTimers::*;

    #[derive(Debug, PartialEq)]
    enum ServiceTimers {
        Period10ms(usize),
        Period15ms(usize),
        Timeout20ms(usize),
        Timeout30ms(usize),
    }
    impl GetMillis for ServiceTimers {
        fn get_millis(&self) -> std::time::Duration {
            match self {
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

    #[test]
    fn new_should_create_empty_queue() {
        let timer_queue = TimerQueue::<ServiceTimers, 10>::new();
        assert!(timer_queue.is_empty())
    }

    #[test]
    fn push_should_insert_timer_according_to_deadline() {
        let mut timer_queue = TimerQueue::<ServiceTimers, 10>::new();
        timer_queue.push(Timer::init(Period15ms(0)));
        timer_queue.push(Timer::init(Timeout30ms(0)));
        timer_queue.push(Timer::init(Timeout20ms(0)));
        timer_queue.push(Timer::init(Period10ms(0)));
        let v: Vec<_> = timer_queue.into();
        assert_eq!(
            v,
            vec![Timeout30ms(0), Timeout20ms(0), Period15ms(0), Period10ms(0)]
        )
    }

    #[test]
    fn pop_should_remove_the_earliest_timer() {
        let mut timer_queue = TimerQueue::<ServiceTimers, 10>::new();
        timer_queue.push(Timer::init(Period15ms(0)));
        timer_queue.push(Timer::init(Timeout30ms(0)));
        timer_queue.push(Timer::init(Timeout20ms(0)));
        timer_queue.push(Timer::init(Period10ms(0)));
        timer_queue.println();
        assert!(timer_queue.len() == 4);
        assert_eq!(
            timer_queue.pop(),
            Some(Timer::from_millis(10, Period10ms(0)))
        );
        assert!(timer_queue.len() == 3);
        timer_queue.push(Timer::init(Period10ms(1)));
        assert!(timer_queue.len() == 4);
        assert_eq!(timer_queue.pop(), Some(Timer::from_millis(5, Period15ms(0))));
        assert!(timer_queue.len() == 3);
        assert_eq!(
            timer_queue.pop(),
            Some(Timer::from_millis(5, Timeout20ms(0)))
        );
        assert!(timer_queue.len() == 2);
        assert_eq!(timer_queue.pop(), Some(Timer::from_millis(0, Period10ms(1))));
        assert!(timer_queue.len() == 1);
        assert_eq!(
            timer_queue.pop(),
            Some(Timer::from_millis(10, Timeout30ms(0)))
        );
        assert!(timer_queue.len() == 0);
        assert_eq!(timer_queue.pop(), None);
        assert!(timer_queue.len() == 0);
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
        timer_queue: TimerQueue<ServiceTimers, 10>,
        timers: HashMap<usize, TimerInfos>,
        fresh_id: usize,
        global_time: Duration,
    }
    impl TimersManager {
        fn new() -> Self {
            let timer_queue = TimerQueue::<ServiceTimers, 10>::new();
            TimersManager {
                timer_queue,
                timers: Default::default(),
                fresh_id: 0,
                global_time: Duration::from_millis(0),
            }
        }
        fn insert_timer(&mut self, mut timer: ServiceTimers) {
            // if queue is full, do nothing (not the purpose of the test)
            if self.timer_queue.is_full() {
                return;
            }

            timer.set_id(self.fresh_id);
            let timer_infos = TimerInfos::new(timer.get_millis(), self.global_time);

            self.timer_queue.push(Timer::init(timer));
            self.timers.insert(self.fresh_id, timer_infos);

            self.fresh_id += 1;
        }
        fn pop_timer(&mut self) {
            // if queue is empty, do nothing (not the purpose of the test)
            if self.timer_queue.is_empty() {
                return;
            }
            let timer = self.timer_queue.pop().unwrap();
            let timer_id = timer.kind.get_id();
            let timer_infos = self.timers.get(&timer_id).unwrap();

            // asserting that deadlines are respected
            let global_time = self.global_time;
            let timer_popped_deadline = timer.deadline;
            let timer_pushed_time = timer_infos.pushed_time;
            let timer_duration = timer_infos.duration;
            assert!(global_time + timer_popped_deadline == timer_pushed_time + timer_duration);

            // update global time
            self.global_time = global_time + timer_popped_deadline;
        }
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
