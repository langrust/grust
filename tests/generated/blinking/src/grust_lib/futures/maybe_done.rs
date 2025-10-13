use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

/// A future that may have completed.
///
/// This is created by the [`maybe_done()`] function.
#[derive(Debug)]
#[pin_project(project = MaybeDoneProj, project_replace = MaybeDoneProjReplace)]
pub enum MaybeDone<Fut: Future> {
    /// A not-yet-completed future.
    Future(#[pin] Fut),
    /// The output of the completed future
    Done(Fut::Output),
    /// The empty variant after the result of a [`MaybeDone`] has been
    /// taken using the [`take_output`](MaybeDone::take_output) method.
    Gone,
}

impl<Fut: Future> Clone for MaybeDone<Fut>
where
    Fut: Clone,
    Fut::Output: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Future(arg0) => Self::Future(arg0.clone()),
            Self::Done(arg0) => Self::Done(arg0.clone()),
            Self::Gone => Self::Gone,
        }
    }
}

/// Wraps a future into a `MaybeDone`.
pub fn maybe_done<Fut: Future>(future: Fut) -> MaybeDone<Fut> {
    MaybeDone::Future(future)
}

impl<Fut: Future> MaybeDone<Fut> {
    /// Attempt to take the output of a `MaybeDone` without driving it
    /// towards completion.
    #[inline]
    pub fn take_output(self: Pin<&mut Self>) -> Option<Fut::Output> {
        match &*self {
            Self::Done(_) => {}
            Self::Future(_) | Self::Gone => return None,
        }
        match self.project_replace(Self::Gone) {
            MaybeDoneProjReplace::Done(output) => Some(output),
            _ => unreachable!(),
        }
    }
}

impl<Fut: Future> Future for MaybeDone<Fut> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.as_mut().project() {
            MaybeDoneProj::Future(f) => {
                let res = match f.poll(cx) {
                    Poll::Ready(t) => t,
                    Poll::Pending => return Poll::Pending,
                };
                self.set(Self::Done(res));
            }
            MaybeDoneProj::Done(_) => {}
            MaybeDoneProj::Gone => panic!("MaybeDone polled after value taken"),
        }
        Poll::Ready(())
    }
}
