pub fn add(x: i64, y: i64) -> i64 {
    let res = x + y;
    res
}
use counter::{CounterInput, CounterState};
pub struct TestInput {}
pub struct TestState {
    last_not_half: bool,
    last_stop: bool,
    counter: CounterState,
}
impl TestState {
    pub fn init() -> TestState {
        TestState {
            last_not_half: Default::default(),
            last_stop: Default::default(),
            counter: CounterState::init(),
        }
    }
    pub fn step(&mut self, input: TestInput) -> i64 {
        let half = self.last_not_half;
        let x = self.last_stop;
        let y = self.counter.step(CounterInput { res: x, tick: half });
        let stop = y > 35i64;
        let not_half = !half;
        self.last_not_half = not_half;
        self.last_stop = stop;
        y
    }
}
