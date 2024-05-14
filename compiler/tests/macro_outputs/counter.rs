pub struct CounterOInput {
    pub res: bool,
    pub tick: bool,
}
pub struct CounterOState {
    mem_o: i64,
}
impl CounterOState {
    pub fn init() -> CounterOState {
        CounterOState { mem_o: 0 }
    }
    pub fn step(&mut self, input: CounterOInput) -> i64 {
        let inc = if input.tick { 1 } else { 0 };
        let o = if input.res { 0 } else { self.mem_o + inc };
        self.mem_o = o;
        o
    }
}
pub struct TestYInput {}
pub struct TestYState {
    mem_x: bool,
    mem_half: bool,
    counter_o: CounterOState,
}
impl TestYState {
    pub fn init() -> TestYState {
        TestYState {
            mem_x: false,
            mem_half: true,
            counter_o: CounterOState::init(),
        }
    }
    pub fn step(&mut self, input: TestYInput) -> i64 {
        let x = self.mem_x;
        let half = self.mem_half;
        let y = self.counter_o.step(CounterOInput { res: x, tick: half });
        self.mem_x = y > 35;
        self.mem_half = !half;
        y
    }
}
