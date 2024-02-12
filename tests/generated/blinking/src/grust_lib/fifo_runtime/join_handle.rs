use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::grust_lib::fifo_runtime::task::Task;

/// An owned permission to join on a task (await its termination).
pub struct JoinHandle<F: Future> {
    task: Option<Arc<Task<F>>>,
}
impl<F: Future> JoinHandle<F> {
    pub(super) fn new(task: Arc<Task<F>>) -> JoinHandle<F> {
        JoinHandle { task: Some(task) }
    }
}

/// Error for JoinHandle.
pub struct JoinError;

impl<F: Future> Future for JoinHandle<F> {
    type Output = Result<F::Output, JoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut ret = Poll::Pending;

        // Task should always be set.
        // If it is not, this is due to polling after completion.
        self.task
            .as_ref()
            .expect("polling after `JoinHandle` already completed")
            .try_read_output(&mut ret, cx.waker());

        ret
    }
}
