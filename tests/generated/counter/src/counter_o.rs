pub struct counter_oInput {
    pub res: bool,
    pub tick: bool,
}
pub struct counter_oState {
    mem_o: i64,
}
impl counter_oState {
    pub fn init() -> counter_oState {
        counter_oState { mem_o: 0i64 }
    }
    pub fn step(self, input: counter_oInput) -> (counter_oState, i64) {
        let inc = if input.tick { 1i64 } else { 0i64 };
        let o = if input.res { 0i64 } else { (self.mem_o) + inc };
        (counter_oState { mem_o: o }, o)
    }
}
