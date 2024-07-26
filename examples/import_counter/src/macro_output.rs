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
            mem: false,
            mem_1: true,
            counter: CounterState::init(),
        }
    }
    pub fn step(&mut self, input: TestInput) -> i64 {
        let x = self.mem;
        let half = self.mem_1;
        let y = self.counter.step(CounterInput { res: x, tick: half });
        self.mem = y > 35i64;
        self.mem_1 = !half;
        y
    }
}
