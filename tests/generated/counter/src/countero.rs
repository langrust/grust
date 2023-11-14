pub struct counteroInput {
    pub res: bool,
    pub tick: bool,
}
pub struct counteroState {
    memo: i64,
}
impl counteroState {
    pub fn init() -> counteroState {
        counteroState { memo: 0i64 }
    }
    pub fn step(self, input: counteroInput) -> (counteroState, i64) {
        let inc = if input.tick { 1i64 } else { 0i64 };
        let o = if input.res { 0i64 } else { (self.memo) + inc };
        (counteroState { memo: o }, o)
    }
}
