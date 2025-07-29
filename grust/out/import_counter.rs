pub struct TestInput {
    pub tick: Option<()>,
}
pub struct TestState {
    last_stop: bool,
    counter: utils::CounterState,
}
impl grust::core::Component for TestState {
    type Input = TestInput;
    type Output = i64;
    fn init() -> TestState {
        TestState {
            last_stop: false,
            counter: <utils::CounterState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: TestInput) -> i64 {
        let x = self.last_stop;
        let y = <utils::CounterState as grust::core::Component>::step(
            &mut self.counter,
            utils::CounterInput {
                res: x,
                tick: input.tick,
            },
        );
        let stop = y > 35i64;
        self.last_stop = stop;
        y
    }
}
