use crate::typedefs::Button;
use crate::counter_o::*;
pub struct ResetButtonUserResetRequestInput {
    pub button_state: Button,
    pub period: i64,
    pub reset_limit_1: i64,
    pub reset_limit_2: i64,
}
pub struct ResetButtonUserResetRequestState {
    mem_res: bool,
    mem_user_reset_request: bool,
    counter_o_counter: CounterOState,
}
impl ResetButtonUserResetRequestState {
    pub fn init() -> ResetButtonUserResetRequestState {
        ResetButtonUserResetRequestState {
            mem_res: true,
            mem_user_reset_request: false,
            counter_o_counter: CounterOState::init(),
        }
    }
    pub fn step(
        self,
        input: ResetButtonUserResetRequestInput,
    ) -> (ResetButtonUserResetRequestState, bool) {
        let res = (input.button_state == Button::Pressed) && (self.mem_res);
        let (counter_o_counter, counter) = self
            .counter_o_counter
            .step(CounterOInput {
                res,
                inc: input.period,
            });
        let user_reset_request = match input.button_state {
            Button::Released => self.mem_user_reset_request,
            Button::Pressed => counter >= input.reset_limit_1 + input.reset_limit_2,
        };
        (
            ResetButtonUserResetRequestState {
                mem_res: input.button_state == Button::Released,
                mem_user_reset_request: user_reset_request,
                counter_o_counter,
            },
            user_reset_request,
        )
    }
}
