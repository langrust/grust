#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    On,
    Off,
}
pub fn add(x: i64, y: i64) -> i64 {
    let res = x + y;
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
        let x = (add)(self.mem_, input.i);
        let o = if input.reset { 0 } else { x };
        self.mem_ = o;
        o
    }
}
pub struct AutomatonInput {
    pub switch: bool,
    pub i: i64,
}
pub struct AutomatonState {
    mem_: State,
    mem__1: i64,
    sum: SumState,
}
impl AutomatonState {
    pub fn init() -> AutomatonState {
        AutomatonState {
            mem_: State::Off,
            mem__1: 0,
            sum: SumState::init(),
        }
    }
    pub fn step(&mut self, input: AutomatonInput) -> i64 {
        let state = self.mem_;
        let (next_state, x, o) = match state {
            State::Off => {
                let next_state = if input.switch { State::On } else { state };
                let x = self.mem__1;
                let o = 0;
                (next_state, x, o)
            }
            State::On => {
                let next_state = if input.switch { State::Off } else { state };
                let x = self.sum.step(SumInput {
                    reset: input.switch,
                    i: input.i,
                });
                let o = 10 * x;
                (next_state, x, o)
            }
        };
        self.mem_ = next_state;
        self.mem__1 = x;
        o
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Context {}
impl Context {
    fn init() -> Context {
        Default::default()
    }
}
