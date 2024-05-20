pub fn add(x: i64, y: i64) -> i64 {
    let res = x + y;
    res
}
pub struct CounterInput {
    pub res: bool,
    pub tick: bool,
}
pub struct CounterState {
    mem_: i64,
}
impl CounterState {
    pub fn init() -> CounterState {
        CounterState { mem_: 0 }
    }
    pub fn step(&mut self, input: CounterInput) -> i64 {
        let inc = if input.tick { 1 } else { 0 };
        let o = if input.res { 0 } else { (add)(self.mem_, inc) };
        self.mem_ = o;
        o
    }
}
pub struct TestInput {}
pub struct TestState {
    mem_: bool,
    mem__1: bool,
    counter: CounterState,
}
impl TestState {
    pub fn init() -> TestState {
        TestState {
            mem_: false,
            mem__1: true,
            counter: CounterState::init(),
        }
    }
    pub fn step(&mut self, input: TestInput) -> i64 {
        let x = self.mem_;
        let half = self.mem__1;
        let y = self.counter.step(CounterInput { res: x, tick: half });
        self.mem_ = y > 35;
        self.mem__1 = !half;
        y
    }
}
