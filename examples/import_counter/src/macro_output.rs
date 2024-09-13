pub fn add(x: i64, y: i64) -> i64 {
    let res = x + y;
    res
}
use counter::{CounterInput, CounterState};
pub struct TestInput {}
pub struct TestState {
    mem: bool,
    mem_1: bool,
    counter: CounterState,
}
impl TestState {
    pub fn init() -> TestState {
        TestState {
            mem: Default::default(),
            mem_1: Default::default(),
            counter: CounterState::init(),
        }
    }
    pub fn step(&mut self, input: TestInput) -> i64 {
        let half = self.mem_1;
        let x = self.mem;
        let y = self.counter.step(CounterInput { res: x, tick: half });
        let stop = y > 35i64;
        let not_half = !half;
        self.mem = stop;
        self.mem_1 = not_half;
        y
    }
}
