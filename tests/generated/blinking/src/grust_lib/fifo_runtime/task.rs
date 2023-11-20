use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use crate::grust_lib::fifo_runtime::{arc_waker::{waker, ArcWake}, join_handle::{JoinError, JoinHandle}, spawner::Spawner, state::{Snapshot, State}};

/// Unifying all Tasks under the same behavior.
pub trait Runnable
where
    Self: Send + Sync,
{
    /// Polls the inner future.
    fn poll(self: Arc<Self>);

    /// Tell if it is the end.
    fn is_end(self: Arc<Self>) -> bool;
}

/// Either the future or the output.
pub(super) enum Stage<F: Future> {
    Running(F),
    Finished(Result<F::Output, JoinError>),
    Consumed,
}

/// [Task] structure.
///
/// Contains the future as well as the necessary data to schedule
/// the future once it is woken.
pub struct Task<F: Future> {
    stage: Mutex<Stage<F>>,
    spawner: Spawner,
    state: State,
    waker: Mutex<Option<Waker>>,
    blocking: bool,
}

impl<F: Future> Task<F> {
    /// New task.
    pub fn new(future: F, spawner: &Spawner, blocking: bool) -> Self
    where
        F: Future + Send + 'static,
    {
        Task {
            stage: Mutex::new(Stage::Running(future)),
            spawner: spawner.clone(),
            state: State::new(),
            waker: Mutex::new(None),
            blocking,
        }
    }

    /// Read the task output into `dst`.
    pub(super) fn try_read_output(
        &self,
        dst: &mut Poll<Result<F::Output, JoinError>>,
        waker: &Waker,
    ) {
        // Load a snapshot of the current task state
        let snapshot = self.state.load();

        debug_assert!(snapshot.is_join_interested());

        if !snapshot.is_complete() {
            // The waker must be stored in the task struct.
            let res = if snapshot.has_join_waker() {
                // There already is a waker stored in the struct. If it matches
                // the provided waker, then there is no further work to do.
                // Otherwise, the waker must be swapped.
                let will_wake = self
                    .waker
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .will_wake(waker);

                if will_wake {
                    // The task is not complete **and** the waker is up to date,
                    // there is nothing further that needs to be done.
                    return;
                }

                // Unset the `JOIN_WAKER` to gain mutable access to the `waker`
                // field then update the field with the new join worker.
                //
                // This requires two atomic operations, unsetting the bit and
                // then resetting it. If the task transitions to complete
                // concurrently to either one of those operations, then setting
                // the join waker fails and we proceed to reading the task
                // output.
                self.state
                    .unset_waker()
                    .and_then(|snapshot| self.set_join_waker(waker.clone(), snapshot))
            } else {
                self.set_join_waker(waker.clone(), snapshot)
            };

            match res {
                Ok(_) => return,
                Err(snapshot) => {
                    assert!(snapshot.is_complete());
                }
            }
        }

        *dst = Poll::Ready(self.take_output());
    }

    fn set_join_waker(&self, waker: Waker, snapshot: Snapshot) -> Result<Snapshot, Snapshot> {
        assert!(snapshot.is_join_interested());
        assert!(!snapshot.has_join_waker());

        let mut guard = self.waker.lock().unwrap();

        // Safety: Only the `JoinHandle` may set the `waker` field. When
        // `JOIN_INTEREST` is **not** set, nothing else will touch the field.
        *guard.deref_mut() = Some(waker);

        // Update the `JoinWaker` state accordingly
        let res = self.state.set_join_waker();

        // If the state could not be updated, then clear the join waker
        if res.is_err() {
            *guard.deref_mut() = None;
        }

        res
    }

    /// Store the task output.
    pub fn store_output(&self, output: Result<F::Output, JoinError>) {
        let mut guard = self.stage.lock().unwrap();
        *guard.deref_mut() = Stage::Finished(output);
    }

    /// Take the task output.
    fn take_output(&self) -> Result<F::Output, JoinError> {
        use std::mem;

        let mut guard = self.stage.lock().unwrap();
        match mem::replace(guard.deref_mut(), Stage::Consumed) {
            Stage::Finished(output) => output,
            _ => panic!("unexpected task state"),
        }
    }

    /// Drop the future.
    pub(super) fn drop_future_or_output(&self) {
        let mut guard = self.stage.lock().unwrap();
        *guard.deref_mut() = Stage::Consumed;
    }

    fn cancel_task(&self) {
        // Drop the future from a panic guard.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.drop_future_or_output();
        }));

        self.complete(Err(JoinError), true);
    }

    fn wake_join(&self) {
        let guard = self.waker.lock().unwrap();
        match guard.deref() {
            Some(waker) => waker.wake_by_ref(),
            None => panic!("waker missing"),
        };
    }

    /// Transitions the task's lifecycle to `Complete`. Notifies the
    /// `JoinHandle` if it still has interest in the completion.
    fn transition_to_complete(&self) {
        // Transition the task's lifecycle to `Complete` and get a snapshot of
        // the task's sate.
        let snapshot = self.state.transition_to_complete();

        if !snapshot.is_join_interested() {
            // The `JoinHandle` is not interested in the output of this task. It
            // is our responsibility to drop the output.
            self.drop_future_or_output();
        } else if snapshot.has_join_waker() {
            // Notify the join handle. The previous transition obtains the
            // lock on the waker cell.
            self.wake_join();
        }
    }

    fn complete(&self, output: Result<F::Output, JoinError>, is_join_interested: bool) {
        if is_join_interested {
            // Store the output. The future has already been dropped
            self.store_output(output);

            // Transition to `Complete`, notifying the `JoinHandle` if necessary.
            self.transition_to_complete();
        }
    }
}

/// Create a new task with an associated join handle
pub(crate) fn joinable<F>(
    future: F,
    spawner: &Spawner,
    blocking: bool,
) -> (Arc<Task<F>>, JoinHandle<F>)
where
    F: Future + Send + 'static,
{
    let task: Arc<Task<F>> = Task::new(future, spawner, blocking).into();

    let join = JoinHandle::new(task.clone());

    (task, join)
}

impl<F> Runnable for Task<F>
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    fn poll(self: Arc<Self>) {
        // Transition the task to the running state.
        //
        // A failure to transition here indicates the task has been cancelled
        // while in the run queue pending execution.
        let snapshot = match self.state.transition_to_running(false) {
            Ok(snapshot) => snapshot,
            Err(_) => {
                // The task was shutdown while in the run queue.
                return;
            }
        };

        // The transition to `Running` done above ensures that a lock on the
        // future has been obtained. This also ensures the `*mut T` pointer
        // contains the future (as opposed to the output) and is initialized.
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // struct Guard<'a, T: Future, S: Schedule> {
            //     core: &'a Core<T, S>,
            // }

            // impl<T: Future, S: Schedule> Drop for Guard<'_, T, S> {
            //     fn drop(&mut self) {
            //         self.core.drop_future_or_output();
            //     }
            // }

            // let guard = Guard { core: self.core() };

            // If the task is cancelled, avoid polling it, instead signalling it
            // is complete.
            if snapshot.is_cancelled() {
                Poll::Ready(Err(JoinError))
            } else {
                let res = {
                    let mut guard = self.stage.lock().unwrap();
                    // Safety: The caller ensures mutual exclusion to the field.
                    let future = match guard.deref_mut() {
                        Stage::Running(future) => future,
                        _ => unreachable!("unexpected stage"),
                    };

                    // Safety: The caller ensures the future is pinned.
                    let future = unsafe { Pin::new_unchecked(future) };

                    // The waker passed into the `poll` function does not require a ref
                    // count increment.
                    // Get a waker referencing the task.
                    let waker = waker(self.clone());
                    // Initialize the task context with the waker.
                    let mut cx = Context::from_waker(&waker);

                    future.poll(&mut cx)
                };

                if res.is_ready() {
                    self.drop_future_or_output();
                }

                res.map(Ok)
            }
        }));

        match res {
            Ok(Poll::Ready(out)) => {
                self.complete(out, snapshot.is_join_interested());
            }
            Ok(Poll::Pending) => {
                match self.state.transition_to_idle() {
                    Ok(snapshot) => {
                        if snapshot.is_notified() {
                            // Signal yield
                            self.spawner.sender.send(self.clone()).unwrap();
                        }
                    }
                    Err(_) => self.cancel_task(),
                }
            }
            Err(_) => {
                self.complete(Err(JoinError), snapshot.is_join_interested());
            }
        }
    }

    fn is_end(self: Arc<Self>) -> bool {
        let guard = self.stage.lock().unwrap();
        if let Stage::Finished(_) = guard.deref() {
            self.blocking
        } else {
            false
        }
    }
}

// The standard library provides low-level, unsafe  APIs for defining wakers.
// Instead of writing unsafe code, we will use the helpers provided by the
// `futures` crate to define a waker that is able to schedule our `Task`
// structure.
impl<F> ArcWake for Task<F>
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Schedule the task for execution. The executor receives from the
        // channel and polls tasks.
        if arc_self.state.transition_to_notified() {
            let _ = arc_self.spawner.sender.send(arc_self.clone());
        }
    }
}
