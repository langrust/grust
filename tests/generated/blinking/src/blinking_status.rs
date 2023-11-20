use pin_project::pin_project;
use std::{future::Future, pin::Pin, task::Poll};
use futures::future::Shared;

use crate::{counter_o::*, grust_lib::{futures::{JoinError, maybe_done::MaybeDone}, node::Node, component::Component}};

pub struct BlinkingStatusInput {
    pub tick_number: i64,
}
pub struct BlinkingStatusState {
    mem_on_off: bool,
    mem_res: bool,
    counter_o_counter: CounterOState,
}
impl Node<BlinkingStatusInput, i64> for BlinkingStatusState {
    fn init() -> BlinkingStatusState {
        BlinkingStatusState {
            mem_on_off: true,
            mem_res: true,
            counter_o_counter: CounterOState::init(),
        }
    }
    fn step(self, input: BlinkingStatusInput) -> (BlinkingStatusState, i64) {
        let res = self.mem_res;
        let x = true;
        let (counter_o_counter, counter) =
            self.counter_o_counter.step(CounterOInput { res, tick: x });
        let on_off = |t: bool, b: bool| -> bool {
            if t {
                !b
            } else {
                b
            }
        }(res, self.mem_on_off);
        let status = if on_off { counter + 1i64 } else { 0i64 };
        (
            BlinkingStatusState {
                mem_on_off: on_off,
                mem_res: (counter + 1i64 == input.tick_number),
                counter_o_counter,
            },
            status,
        )
    }
}

impl Component<BlinkingStatusInput, i64> for BlinkingStatusState {}

#[pin_project]
pub struct BlinkingStatusAsyncInput<F1>
where
    F1: Future<Output = Result<i64, JoinError>> + Send + 'static,
{
    /// Input `tick_number`.
    #[pin]
    pub tick_number: MaybeDone<Shared<F1>>,
}

impl<F1> Future for BlinkingStatusAsyncInput<F1>
where
    F1: Future<Output = Result<i64, JoinError>> + Send + 'static,
{
    type Output = Result<BlinkingStatusInput, JoinError>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut all_done = true;
        let mut futures = self.project();
        all_done &= futures.tick_number.as_mut().poll(cx).is_ready();

        if all_done {
            Poll::Ready(Ok(BlinkingStatusInput {
                tick_number: futures.tick_number.take_output().unwrap()?,
            }))
        } else {
            Poll::Pending
        }
    }
}
