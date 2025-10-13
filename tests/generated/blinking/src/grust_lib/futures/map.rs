use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

/// A future that will be mapped by a function when ready.
///
/// This is created by the [`map()`] function.
#[derive(Debug)]
#[pin_project(project = MapProj, project_replace = MapProjReplace)]
pub enum Map<T, Fut: Future> {
    /// A not-yet-completed future.
    Future(#[pin] Fut, fn(Fut::Output) -> T),
    /// The empty variant after the ready [`Map`] is polled.
    Gone,
}

impl<T, Fut: Future> Clone for Map<T, Fut>
where
    Fut: Clone,
    Fut::Output: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Future(arg0, arg1) => Self::Future(arg0.clone(), arg1.clone()),
            Self::Gone => Self::Gone,
        }
    }
}

/// Wraps a future and a function into a `Map`.
pub fn map<T, Fut: Future>(future: Fut, function: fn(Fut::Output) -> T) -> Map<T, Fut> {
    Map::Future(future, function)
}

impl<T, Fut: Future> Future for Map<T, Fut> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.as_mut().project() {
            MapProj::Future(future, ..) => {
                let res = match future.poll(cx) {
                    Poll::Ready(t) => t,
                    Poll::Pending => return Poll::Pending,
                };
                match self.project_replace(Map::Gone) {
                    MapProjReplace::Future(_, function) => Poll::Ready(function(res)),
                    MapProjReplace::Gone => unreachable!(),
                }
            }
            MapProj::Gone => panic!("Map polled after value taken"),
        }
    }
}
