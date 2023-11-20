#![warn(missing_docs)]
//! A very basic futures [FIFORuntime](fifo_runtime::FIFORuntime) based on a channel.
//! When tasks are woken, they are scheduled by queuing them in the send half of
//! the channel. The [Executor](fifo_runtime::Executor) waits on the receive half
//! and executes received tasks.
//!
//! When a task is executed, the send half of the channel, the [Spawner](fifo_runtime::Spawner),
//! is passed along via the task's [Waker].

/// ArcWaker module.
pub mod arc_waker;
/// Context module.
pub mod context;
/// Executor module.
pub mod executor;
/// JoinHandle module.
pub mod join_handle;
/// SpawnHandle module.
pub mod spawn_handle;
/// Spawner module.
pub mod spawner;
/// State module.
pub mod state;
/// Task module.
pub mod task;

use std::future::Future;
use std::sync::mpsc::sync_channel;

use executor::Executor;
use spawn_handle::SpawnHandle;
use spawner::Spawner;

pub use context::spawn;

/// A very basic futures [FIFORuntime] based on a channel. When tasks are woken, they
/// are scheduled by queuing them in the send half of the channel. The [Executor]
/// waits on the receive half and executes received tasks.
///
/// When a task is executed, the send half of the channel, the [Spawner], is passed
/// along via the task's [Waker].
pub struct FIFORuntime {
    /// Receives scheduled tasks. When a task is scheduled, the associated future
    /// is ready to make progress. This usually happens when a resource the task
    /// uses becomes ready to perform an operation. For example, a socket
    /// received data and a `read` call will succeed.
    pub executor: Executor,

    /// Send half of the scheduled channel.
    pub spawner: Spawner,
}
impl FIFORuntime {
    /// Initialize a new fifo-runtime instance.
    pub fn new() -> FIFORuntime {
        const MAX_QUEUED_TASKS: usize = 10_000;
        let (sender, scheduled) = sync_channel(MAX_QUEUED_TASKS);
        let executor = Executor { scheduled };
        let spawner = Spawner { sender };

        let spawn_handle = SpawnHandle {
            spawner: spawner.clone(),
        };

        spawn_handle.enter(|| println!("FIFO initialized"));

        FIFORuntime { executor, spawner }
    }

    /// Run a future to completion on the runtime.
    pub fn block_on<F>(&mut self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        // Spawn the root task. All other tasks are spawned from the context of this
        // root task. No work happens until `executor.run()` is called.
        self.spawner.spawn(future, true);

        // Start the fifo-runtime executor loop. Scheduled tasks are received and
        // executed.
        self.executor.run();
    }
}
