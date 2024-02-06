use crate::typedefs::Button;
use crate::counter_o::*;
use crate::functions::reset_state_management;
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
    pub fn step(&mut self, input: ResetButtonUserResetRequestStateInput) -> ResetState {
        let res = (input.button_state == Button::Pressed) && (self.mem_res);
        let counter = self
            .counter_o_counter
            .step(CounterOInput {
                res: res,
                inc: input.period,
            });
        let user_reset_request_state = Expr::FunctionCall(
            parse_quote! {
                reset_state_management(input.button_state, counter, input.reset_limit_1)
            },
        );
        self.mem_res = input.button_state == Button::Released;
        user_reset_request_state
    }
}
