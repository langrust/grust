use core::mem;
use core::task::{RawWaker, RawWakerVTable};
use std::sync::Arc;
use std::task::Waker;

/// Creates a [`Waker`] from an `Arc<impl ArcWaker>`.
///
/// The returned [`Waker`] will call
/// [`ArcWake.wake()`](ArcWake::wake) if awoken.
pub fn waker<T>(wake: Arc<T>) -> Waker
where
    T: ArcWake + 'static,
{
    // clone pointer to task
    let ptr = Arc::into_raw(wake).cast::<()>();

    // put it into new waker
    unsafe { Waker::from_raw(RawWaker::new(ptr, waker_vtable::<T>())) }
}

fn waker_vtable<W: ArcWake>() -> &'static RawWakerVTable {
    unsafe fn clone_arc_raw<T: ArcWake>(data: *const ()) -> RawWaker {
        // Retain Arc, but don't touch refcount by wrapping in ManuallyDrop
        let arc = mem::ManuallyDrop::new(Arc::<T>::from_raw(data.cast::<T>()));
        // Now increase refcount, but don't drop new refcount either
        let _arc_clone: mem::ManuallyDrop<_> = arc.clone();
        RawWaker::new(data, waker_vtable::<T>())
    }

    unsafe fn wake_arc_raw<T: ArcWake>(data: *const ()) {
        let arc: Arc<T> = Arc::from_raw(data.cast::<T>());
        ArcWake::wake(arc);
    }

    unsafe fn wake_by_ref_arc_raw<T: ArcWake>(data: *const ()) {
        // Retain Arc, but don't touch refcount by wrapping in ManuallyDrop
        let arc = mem::ManuallyDrop::new(Arc::<T>::from_raw(data.cast::<T>()));
        ArcWake::wake_by_ref(&arc);
    }

    unsafe fn drop_arc_raw<T: ArcWake>(data: *const ()) {
        drop(Arc::<T>::from_raw(data.cast::<T>()))
    }

    &RawWakerVTable::new(
        clone_arc_raw::<W>,
        wake_arc_raw::<W>,
        wake_by_ref_arc_raw::<W>,
        drop_arc_raw::<W>,
    )
}

/// A way of waking up a specific task.
///
/// By implementing this trait, types that are expected to be wrapped in an `Arc`
/// can be converted into [`Waker`] objects.
/// Those Wakers can be used to signal executors that a task it owns
/// is ready to be `poll`ed again.
///
/// Currently, there are two ways to convert `ArcWake` into [`Waker`]:
///
/// * [`waker`](super::waker()) converts `Arc<impl ArcWake>` into [`Waker`].
/// * [`waker_ref`](super::waker_ref()) converts `&Arc<impl ArcWake>` into [`WakerRef`] that
///   provides access to a [`&Waker`][`Waker`].
///
/// [`Waker`]: std::task::Waker
/// [`WakerRef`]: super::WakerRef
// Note: Send + Sync required because `Arc<T>` doesn't automatically imply
// those bounds, but `Waker` implements them.
pub trait ArcWake: Send + Sync {
    /// Indicates that the associated task is ready to make progress and should
    /// be `poll`ed.
    ///
    /// This function can be called from an arbitrary thread, including threads which
    /// did not create the `ArcWake` based [`Waker`].
    ///
    /// Executors generally maintain a queue of "ready" tasks; `wake` should place
    /// the associated task onto this queue.
    ///
    /// [`Waker`]: std::task::Waker
    fn wake(self: Arc<Self>) {
        Self::wake_by_ref(&self)
    }

    /// Indicates that the associated task is ready to make progress and should
    /// be `poll`ed.
    ///
    /// This function can be called from an arbitrary thread, including threads which
    /// did not create the `ArcWake` based [`Waker`].
    ///
    /// Executors generally maintain a queue of "ready" tasks; `wake_by_ref` should place
    /// the associated task onto this queue.
    ///
    /// This function is similar to [`wake`](ArcWake::wake), but must not consume the provided data
    /// pointer.
    ///
    /// [`Waker`]: std::task::Waker
    fn wake_by_ref(arc_self: &Arc<Self>);
}
