use crate::counter_o::*;
pub struct MainYInput {}
pub struct MainYState {
    mem_x: bool,
    mem_half: bool,
    counter_o: CounterOState,
}
impl MainYState {
    pub fn init() -> MainYState {
        MainYState {
            mem_x: false,
            mem_half: true,
            counter_o: CounterOState::init(),
        }
    }
    pub fn step(&mut self, input: MainYInput) -> i64 {
        let x = self.mem_x;
        let half = self.mem_half;
        let y = self
            .counter_o
            .step(CounterOInput {
                res: x,
                tick: half,
            });
        self.mem_x = (y > 35i64);
        self.mem_half = !half;
        y
    }
}
