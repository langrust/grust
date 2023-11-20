use crate::typedefs::Button;
use crate::counter_o::*;
use crate::typedefs::ResetState;
pub struct ResetButtonUserResetRequestStateInput {
    pub button_state: Button,
    pub period: i64,
    pub reset_limit_1: i64,
}
pub struct ResetButtonUserResetRequestStateState {
    mem_res: bool,
    counter_o_counter: CounterOState,
}
impl ResetButtonUserResetRequestStateState {
    pub fn init() -> ResetButtonUserResetRequestStateState {
        ResetButtonUserResetRequestStateState {
            mem_res: true,
            counter_o_counter: CounterOState::init(),
        }
    }
    pub fn step(
        self,
        input: ResetButtonUserResetRequestStateInput,
    ) -> (ResetButtonUserResetRequestStateState, ResetState) {
        let res = (input.button_state == Button::Pressed) && (self.mem_res);
        let (counter_o_counter, counter) = self
            .counter_o_counter
            .step(CounterOInput {
                res,
                inc: input.period,
            });
        let user_reset_request_state = match input.button_state {
            Button::Released => ResetState::Confirmed,
            Button::Pressed => {
                if counter >= input.reset_limit_1 {
                    ResetState::InProgress
                } else {
                    ResetState::Confirmed
                }
            }
        };
        (
            ResetButtonUserResetRequestStateState {
                mem_res: input.button_state == Button::Released,
                counter_o_counter,
            },
            user_reset_request_state,
        )
    }
}
