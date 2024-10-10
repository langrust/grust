use std::{cmp::Ordering, fmt::Debug};

pub struct PrioQueue<T, F, const N: usize>
where
    F: FnMut(&T, &T) -> Ordering,
{
    queue: [Option<T>; N],
    order: F,
    len: usize,
}
impl<T, F, const N: usize> PrioQueue<T, F, N>
where
    F: FnMut(&T, &T) -> Ordering,
{
    /// Create empty queue.
    pub fn new(order: F) -> Self {
        PrioQueue {
            queue: array_init::array_init(|_| None),
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
            let curr = self.queue[index].as_ref().unwrap();
            match (self.order)(&value, curr) {
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
    /// Pop the smallest element of the queue.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let res = std::mem::take(&mut self.queue[self.len - 1]);
            self.len -= 1;
            res
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
impl<T, F, const N: usize> PrioQueue<T, F, N>
where
    F: FnMut(&T, &T) -> Ordering,
    T: PartialEq,
{
    /// Reset a value in the queue.
    ///
    /// This will remove the previous version of the value and add the new one.
    /// This will push the value if not in the queue.
    ///
    /// # Panics
    ///
    /// This function will panic if the queue is full.
    pub fn reset(&mut self, value: T) {
        // removes resetted timers
        for index in (0..self.len).rev() {
            let curr = self.queue[index].as_mut().unwrap();

            // if curr should be resetted then remove it
            // and add its dealine to the next timer (if it exists)
            if value.eq(curr) {
                self.queue[index] = None;
                self.queue[index..self.len].rotate_left(1);
                self.len -= 1;
            }
        }
        // pushes the value at the right place
        self.push(value)
    }
}
impl<T, F, const N: usize> Into<Vec<T>> for PrioQueue<T, F, N>
where
    F: FnMut(&T, &T) -> Ordering,
{
    fn into(self) -> Vec<T> {
        let v = self
            .queue
            .into_iter()
            .take(self.len)
            .map(|opt| opt.unwrap())
            .collect::<Vec<_>>();
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
    fn push_should_insert_duplicate() {
        let mut prio_queue = PrioQueue::<_, _, 10>::new(order);
        prio_queue.push(3);
        prio_queue.push(4);
        prio_queue.push(-1);
        prio_queue.push(2);
        prio_queue.push(4);
        prio_queue.push(5);
        let v: Vec<_> = prio_queue.into();
        assert_eq!(v, vec![5, 4, 4, 3, 2, -1])
    }

    #[test]
    fn reset_should_not_insert_duplicate() {
        let mut prio_queue = PrioQueue::<_, _, 10>::new(order);
        prio_queue.push(3);
        prio_queue.push(4);
        prio_queue.push(-1);
        prio_queue.push(2);
        prio_queue.push(4);
        prio_queue.push(5);
        prio_queue.reset(4);
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
