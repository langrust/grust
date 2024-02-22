use crate::typedefs::Button;
use crate::counter_o::*;
use crate::functions::reset_state_management;
use crate::typedefs::ResetState;
pub struct ResetButtonUserResetRequestStateInput {
    pub button_state: Button,
    pub period: i64,
    pub reset_limit_1: i64,
}
pub struct ResetButtonUserResetRequestStateState {
    mem_res: bool,
    counter_o: CounterOState,
}
impl ResetButtonUserResetRequestStateState {
    pub fn init() -> ResetButtonUserResetRequestStateState {
        ResetButtonUserResetRequestStateState {
            mem_res: true,
            counter_o: CounterOState::init(),
        }
    }
    pub fn step(&mut self, input: ResetButtonUserResetRequestStateInput) -> ResetState {
        let res = (input.button_state == Button::Pressed) && (self.mem_res);
        let counter = self
            .counter_o
            .step(CounterOInput {
                res: res,
                inc: input.period,
            });
        let user_reset_request_state = reset_state_management(
            input.button_state,
            counter,
            input.reset_limit_1,
        );
        self.mem_res = input.button_state == Button::Released;
        user_reset_request_state
    }
}
