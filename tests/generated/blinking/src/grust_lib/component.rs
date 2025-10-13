use futures::{future::Shared, FutureExt};
use std::future::Future;

use crate::grust_lib::{
    node::Node, fifo_runtime::{self, join_handle::JoinHandle},
    futures::JoinError,
};

use super::futures::{async_step::{AsyncStep, async_step as future_async_step}, map::{Map, map}, ready::{Ready, ready}};

/// Component related behaviors.
pub trait Component<I, O>
where
    Self: Node<I, O> + Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
    I: Send + 'static,
{
    /// Initialize a component state.
    fn async_init(
    ) -> Map<Result<Self, JoinError>,JoinHandle<Ready<Self>>>
    {
        map(fifo_runtime::spawn(ready(Self::init())), JoinError::convert_error)
    }
    
    /// Perform the step of a state with its input.
    fn async_step<FutState, FutInput>(
        state: FutState, 
        input: FutInput,
    ) -> (
        Map<Result<Self, JoinError>, Shared<Map<Result<(Self, O), JoinError>, JoinHandle<AsyncStep<Self, I, O, FutState, FutInput>>>>>,
        Shared<Map<Result<O, JoinError>, Shared<Map<Result<(Self, O), JoinError>, JoinHandle<AsyncStep<Self, I, O, FutState, FutInput>>>>>>
    )
    where
        FutState: Future<Output = Result<Self, JoinError>> + Send + 'static,
        FutInput: Future<Output = Result<I, JoinError>> + Send + 'static,
    {
        let result = map(fifo_runtime::spawn(future_async_step(state, input)), JoinError::convert_error);
        destructure(result)
    }
}


fn destructure<Fut, S, V>(
    future: Fut,
) -> (
    Map<Result<S, JoinError>, Shared<Fut>>,
    Shared<Map<Result<V, JoinError>, Shared<Fut>>>,
)
where
    Fut: Future<Output = Result<(S, V), JoinError>> + Send + 'static,
    S: Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    fn first<S, V>(result: Result<(S, V), JoinError>) -> Result<S, JoinError> {
        result.map(|(s, _)| s)
    }
    fn second<S, V>(result: Result<(S, V), JoinError>) -> Result<V, JoinError> {
        result.map(|(_, v)| v)
    }

    let cloneable = future.shared();
    let cloned = cloneable.clone();

    (map(cloned, first), map(cloneable, second).shared())
}
