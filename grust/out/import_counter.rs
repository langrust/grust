pub struct TestInput {
    pub tick: Option<()>,
}
pub struct TestOutput {
    pub y: i64,
}
pub struct TestState {
    last_stop: bool,
    counter: utils::CounterState,
}
impl grust::core::Component for TestState {
    type Input = TestInput;
    type Output = TestOutput;
    fn init() -> TestState {
        TestState {
            last_stop: false,
            counter: <utils::CounterState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: TestInput) -> TestOutput {
        let x = self.last_stop;
        let y = {
            let utils::CounterOutput { o } = <utils::CounterState as grust::core::Component>::step(
                &mut self.counter,
                utils::CounterInput {
                    res: x,
                    tick: input.tick,
                },
            );
            (o)
        };
        let stop = y > 35i64;
        self.last_stop = stop;
        TestOutput { y }
    }
}
