pub struct CounterOInput {
    pub res: bool,
    pub tick: bool,
}
pub struct CounterOState {
    mem_o: i64,
}
impl CounterOState {
    pub fn init() -> CounterOState {
        CounterOState { mem_o: 0i64 }
    }
    pub fn step(self, input: CounterOInput) -> (CounterOState, i64) {
        let inc = if input.tick { 1i64 } else { 0i64 };
        let o = if input.res { 0i64 } else { (self.mem_o) + inc };
        (CounterOState { mem_o: o }, o)
    }
}
