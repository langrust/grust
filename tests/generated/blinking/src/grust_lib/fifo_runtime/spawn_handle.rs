use std::future::Future;

use crate::grust_lib::fifo_runtime::{context::enter, join_handle::JoinHandle, spawner::Spawner};

/// Handle to the runtime.
#[derive(Debug, Clone)]
pub struct SpawnHandle {
    pub(crate) spawner: Spawner,
}

impl SpawnHandle {
    /// Enter the runtime context.
    pub fn enter<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        enter(self.clone(), f)
    }

    /// Spawns a new asynchronous task.
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F>
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        self.spawner.spawn(future, false)
    }
}
