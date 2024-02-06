use crate::counter_o::*;
use crate::functions::xor;
pub struct BlinkingStatusInput {
    pub tick_number: i64,
}
pub struct BlinkingStatusState {
    mem_on_off: bool,
    mem_res: bool,
    counter_o_counter: CounterOState,
}
impl BlinkingStatusState {
    pub fn init() -> BlinkingStatusState {
        BlinkingStatusState {
            mem_on_off: true,
            mem_res: true,
            counter_o_counter: CounterOState::init(),
        }
    }
    pub fn step(&mut self, input: BlinkingStatusInput) -> i64 {
        let res = self.mem_res;
        let x = true;
        let counter = self.counter_o_counter.step(CounterOInput { res: res, tick: x });
        let on_off = Expr::FunctionCall(
            parse_quote! {
                xor(res, self.mem_on_off)
            },
        );
        let status = if on_off { counter + 1i64 } else { 0i64 };
        self.mem_on_off = on_off;
        self.mem_res = (counter + 1i64 == input.tick_number);
        status
    }
}
