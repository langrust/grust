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
        CounterState { mem: 0i64 }
    }
    pub fn step(&mut self, input: CounterInput) -> i64 {
        let inc = if input.tick { 1i64 } else { 0i64 };
        let o = if input.res { 0i64 } else { add(self.mem, inc) };
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
        let stream_expression_fresh_ident = self.mem;
        let half = self.mem_1;
        let y = self.counter.step(CounterInput {
            res: stream_expression_fresh_ident,
            tick: half,
        });
        self.mem = y > 35i64;
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
