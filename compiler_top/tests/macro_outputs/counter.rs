pub fn add(x: i64, y: i64) -> i64 {
    let res = x + y;
    res
}
pub struct CounterInput {
    pub res: bool,
    pub tick: bool,
}
pub struct CounterState {
    last_o: i64,
}
impl grust::core::Component for CounterState {
    type Input = CounterInput;
    type Output = i64;
    fn init() -> CounterState {
        CounterState { last_o: 0i64 }
    }
    fn step(&mut self, input: CounterInput) -> i64 {
        let inc = if input.tick { 1i64 } else { 0i64 };
        let o = if input.res {
            0i64
        } else {
            add(self.last_o, inc)
        };
        self.last_o = o;
        o
    }
}
pub struct TestInput {}
pub struct TestState {
    last_not_half: bool,
    last_stop: bool,
    counter: CounterState,
}
impl grust::core::Component for TestState {
    type Input = TestInput;
    type Output = i64;
    fn init() -> TestState {
        TestState {
            last_not_half: false,
            last_stop: false,
            counter: <CounterState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: TestInput) -> i64 {
        let half = self.last_not_half;
        let x = self.last_stop;
        let y = <CounterState as grust::core::Component>::step(
            &mut self.counter,
            CounterInput { res: x, tick: half },
        );
        let stop = y > 35i64;
        let not_half = !(half);
        self.last_not_half = not_half;
        self.last_stop = stop;
        y
    }
}
