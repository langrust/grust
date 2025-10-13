use std::future::Future;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;

use crate::grust_lib::fifo_runtime::{task::{joinable, Runnable}, join_handle::JoinHandle};

/// [Spawner] spawns new futures onto the task channel.
#[derive(Debug, Clone)]
pub struct Spawner {
    pub(crate) sender: SyncSender<Arc<dyn Runnable>>,
}
impl Spawner {
    /// Spawn a future onto the fifo-runtime instance.
    pub(crate) fn spawn<F>(&self, future: F, blocking: bool) -> JoinHandle<F>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (task, handle) = joinable(future, self, blocking);
        self.sender.send(task).unwrap();
        handle
    }
}
