pub struct TestInput {
    pub tick: Option<()>,
}
pub struct TestState {
    last_stop: bool,
    counter: utils::CounterState,
}
impl TestState {
    pub fn init() -> TestState {
        TestState {
            last_stop: false,
            counter: utils::CounterState::init(),
        }
    }
    pub fn step(&mut self, input: TestInput) -> i64 {
        let x = self.last_stop;
        let y = self.counter.step(utils::CounterInput {
            res: x,
            tick: input.tick,
        });
        let stop = y > 35i64;
        self.last_stop = stop;
        y
    }
}
