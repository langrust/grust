use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

use crate::grust_lib::{node::Node, futures::{JoinError, maybe_done::{maybe_done, MaybeDone}}};

/// A future that represents the asynchronous step fo a component.
///
/// This is created by the [`async_step()`] function.
#[derive(Debug)]
#[pin_project(project = AsyncStepProj)]
pub enum AsyncStep<S, I, O, FutState, FutInput>
where
    FutState: Future<Output = Result<S, JoinError>> + Send + 'static,
    FutInput: Future<Output = Result<I, JoinError>> + Send + 'static,
    S: Node<I, O> + Send + Sync + 'static,
    O: Send + Sync + 'static,
    I: Send,
{
    /// A not-yet-completed future.
    Future {
        /// Component's state as a future that might be done.
        #[pin]
        state: MaybeDone<FutState>,
        /// Component's input as a future that might be done.
        #[pin]
        input: MaybeDone<FutInput>,
        /// Phantom date for the component's output type.
        _phantom: PhantomData<O>,
    },
    /// The empty variant after the ready [`AsyncStep`] is polled.
    Gone,
}

/// Wraps state and input futures into an `AsyncStep`.
pub fn async_step<S, I, O, FutState, FutInput>(
    state: FutState,
    input: FutInput,
) -> AsyncStep<S, I, O, FutState, FutInput>
where
    FutState: Future<Output = Result<S, JoinError>> + Send + 'static,
    FutInput: Future<Output = Result<I, JoinError>> + Send + 'static,
    S: Node<I, O> + Send + Sync + 'static,
    O: Send + Sync + 'static,
    I: Send,
{
    AsyncStep::Future {
        state: maybe_done(state),
        input: maybe_done(input),
        _phantom: PhantomData,
    }
}

impl<S, I, O, FutState, FutInput> Future for AsyncStep<S, I, O, FutState, FutInput>
where
    FutState: Future<Output = Result<S, JoinError>> + Send + 'static,
    FutInput: Future<Output = Result<I, JoinError>> + Send + 'static,
    S: Node<I, O> + Send + Sync + 'static,
    O: Send + Sync + 'static,
    I: Send,
{
    type Output = (S, O);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut all_done = true;

        match self.as_mut().project() {
            AsyncStepProj::Future {
                mut state,
                mut input,
                _phantom,
            } => {
                all_done &= state.as_mut().poll(cx).is_ready();
                all_done &= input.as_mut().poll(cx).is_ready();
                if all_done {
                    let result = {
                        let state = state.as_mut().take_output().unwrap().unwrap();
                        let input = input.as_mut().take_output().unwrap().unwrap();
                        state.step(input)
                    };
                    self.set(AsyncStep::Gone);
                    Poll::Ready(result)
                } else {
                    Poll::Pending
                }
            }
            AsyncStepProj::Gone => panic!("AsyncStep polled after value taken"),
        }
    }
}
