pub struct CounterOInput {
    pub res: bool,
    pub inc: i64,
}
pub struct CounterOState {
    mem_o: i64,
}
impl CounterOState {
    pub fn init() -> CounterOState {
        CounterOState { mem_o: 0i64 }
    }
    pub fn step(self, input: CounterOInput) -> (CounterOState, i64) {
        let o = if input.res { 0i64 } else { (self.mem_o) + input.inc };
        (CounterOState { mem_o: o }, o)
    }
}
