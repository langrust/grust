use std::sync::mpsc::Receiver;
use std::sync::Arc;

use crate::grust_lib::fifo_runtime::task::Runnable;

/// Task [Executor] that receives tasks off of a channel and runs them.
pub struct Executor {
    pub(crate) scheduled: Receiver<Arc<dyn Runnable>>,
}
impl Executor {
    /// Run the executor.
    ///
    /// This starts the executor loop and runs it indefinitely. No shutdown
    /// mechanism has been implemented.
    ///
    /// Tasks are popped from the `scheduled` channel receiver. Receiving a task
    /// on the channel signifies the task is ready to be executed. This happens
    /// when the task is first created and when its waker has been used.
    pub fn run(&self) {
        // The executor loop. Scheduled tasks are received. If the channel is
        // empty, the thread blocks until a task is received.
        while let Ok(task) = self.scheduled.recv() {
            // Execute the task until it either completes or cannot make further
            // progress and returns `Poll::Pending`.
            task.clone().poll();
            if task.is_end() {
                break;
            }
        }
    }
}
