#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    On,
    Off,
}
pub fn add(x: i64, y: i64) -> i64 {
    let res: i64 = x + y;
    res
}
pub struct SumInput {
    pub reset: bool,
    pub i: i64,
}
pub struct SumState {
    mem_: i64,
}
impl SumState {
    pub fn init() -> SumState {
        SumState { mem_: 0 }
    }
    pub fn step(&mut self, input: SumInput) -> i64 {
        let x: i64 = (add)(self.mem_, input.i);
        let o = if input.reset { 0 } else { x };
        self.mem_ = o;
        o
    }
}
