use crate::counter_o::*;
use crate::functions::xor;
pub struct BlinkingStatusInput {
    pub tick_number: i64,
}
pub struct BlinkingStatusState {
    mem_res: bool,
    mem_on_off: bool,
    counter_o: CounterOState,
}
impl BlinkingStatusState {
    pub fn init() -> BlinkingStatusState {
        BlinkingStatusState {
            mem_res: true,
            mem_on_off: true,
            counter_o: CounterOState::init(),
        }
    }
    pub fn step(&mut self, input: BlinkingStatusInput) -> i64 {
        let x = true;
        let res = self.mem_res;
        let counter = self.counter_o.step(CounterOInput { res: res, tick: x });
        let on_off = xor(res, self.mem_on_off);
        let status = if on_off { counter + 1i64 } else { 0i64 };
        self.mem_res = (counter + 1i64 == input.tick_number);
        self.mem_on_off = on_off;
        status
    }
}
