use std::cell::RefCell;
use std::future::Future;

use crate::grust_lib::fifo_runtime::{join_handle::JoinHandle, spawn_handle::SpawnHandle};

thread_local! {
    static CONTEXT: RefCell<Option<SpawnHandle>> = RefCell::new(None)
}

pub(crate) fn spawn_handle() -> Option<SpawnHandle> {
    CONTEXT.with(|ctx| match *ctx.borrow() {
        Some(ref ctx) => Some(ctx.clone()),
        None => None,
    })
}

/// Set this [`SpawnHandle`] as the current active [`SpawnHandle`].
pub(crate) fn enter<F, R>(new: SpawnHandle, f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = CONTEXT.with(|ctx| {
        let _ = ctx.borrow_mut().replace(new);
    });

    f()
}

/// Spawns a new asynchronous task.
pub fn spawn<F>(future: F) -> JoinHandle<F>
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    let spawn_handle = spawn_handle()
        .expect("must be called from the context of FIFO runtime initialised");
    spawn_handle.spawn(future)
}
