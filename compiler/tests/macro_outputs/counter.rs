pub fn add(x: i64, y: i64) -> i64 {
    let res = x + y;
    res
}
pub struct CounterInput {
    pub res: bool,
    pub tick: bool,
}
pub struct CounterState {
    mem: i64,
}
impl CounterState {
    pub fn init() -> CounterState {
        CounterState { mem: 0 }
    }
    pub fn step(&mut self, input: CounterInput) -> i64 {
        let inc = if input.tick { 1 } else { 0 };
        let o = if input.res { 0 } else { add(self.mem, inc) };
        self.mem = o;
        o
    }
}
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
        self.mem = y > 35;
        self.mem_1 = !half;
        y
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Context {}
impl Context {
    fn init() -> Context {
        Default::default()
    }
}
