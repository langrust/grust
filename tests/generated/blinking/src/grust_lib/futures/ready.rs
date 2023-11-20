use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

/// A future that is immediately ready.
///
/// This is created by the [`ready()`] function.
#[derive(Debug)]
#[pin_project(project_replace = ReadyProjReplace)]
pub enum Ready<T> {
    /// Ready value.
    Value(T),
    /// The empty variant after the ready [`Ready`] is polled.
    Gone,
}

/// Creates a future that is immediately ready with a value.
pub fn ready<T>(value: T) -> Ready<T> {
    Ready::Value(value)
}

impl<T> Future for Ready<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project_replace(Self::Gone) {
            ReadyProjReplace::Value(value) => Poll::Ready(value),
            ReadyProjReplace::Gone => panic!("Ready polled after value taken"),
        }
    }
}
