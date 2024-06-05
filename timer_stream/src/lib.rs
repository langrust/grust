use std::{fmt::Debug, time::Duration};

pub trait GetMillis {
    fn get_millis(&self) -> Duration;
}

#[derive(Default, Debug)]
pub struct Timer<T> {
    remaning: Duration,
    kind: T,
}
impl<T: GetMillis> Timer<T> {
    /// Initiate a new timer.
    pub fn init(kind: T) -> Timer<T> {
        Timer {
            remaning: kind.get_millis(),
            kind,
        }
    }
}
impl<T> Timer<T> {
    /// Consume the timer and returns its kind.
    pub fn get_kind(self) -> T {
        self.kind
    }
}

pub struct TimerQueue<T, const N: usize> {
    queue: [Timer<T>; N],
    len: usize,
}
impl<T, const N: usize> TimerQueue<T, N>
where
    T: Default + Debug,
{
    /// Create empty queue.
    pub fn new() -> Self {
        TimerQueue {
            queue: array_init::array_init(|_| Default::default()),
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
    /// Push a value in timer queue.
    ///
    /// # Panics
    ///
    /// This function will panic if the queue is full.
    pub fn push(&mut self, value: Timer<T>) {
        // safety: panics if pushed out of bound
        if self.is_full() {
            panic!("out of bound")
        }

        // puts the value at the right place
        for index in (0..self.len).rev() {
            if &value.remaning < &self.queue[index].remaning {
                self.queue[index..=self.len].rotate_right(1);
                self.queue[index] = value;
                self.len += 1;
                return;
            } else {

            }
        }
        // if not inserted, then put it at the end
        self.queue[self.len] = value;
        self.len += 1;
    }
    /// Pop the most urgent timer from the queue.
    pub fn pop(&mut self) -> Option<Timer<T>> {
        if self.is_empty() {
            None
        } else {
            let res = std::mem::take(&mut self.queue[self.len - 1]);
            self.len -= 1;
            Some(res)
        }
    }
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
