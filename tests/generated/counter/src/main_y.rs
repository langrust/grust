use crate::counter_o::*;
pub struct MainYInput {}
pub struct MainYState {
    mem_half: bool,
    mem_x: bool,
    counter_o_y: CounterOState,
}
impl MainYState {
    pub fn init() -> MainYState {
        MainYState {
            mem_half: true,
            mem_x: false,
            counter_o_y: CounterOState::init(),
        }
    }
    pub fn step(&mut self, input: MainYInput) -> i64 {
        let half = self.mem_half;
        let x = self.mem_x;
        let y = self
            .counter_o_y
            .step(CounterOInput {
                res: x,
                tick: half,
            });
        self.mem_half = !half;
        self.mem_x = (y > 35i64);
        y
    }
}
