#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum State {
    #[default]
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
    mem: i64,
}
impl SumState {
    pub fn init() -> SumState {
        SumState { mem: 0i64 }
    }
    pub fn step(&mut self, input: SumInput) -> i64 {
        let x = add(self.mem, input.i);
        let o = if input.reset { 0i64 } else { x };
        self.mem = o;
        o
    }
}
pub struct AutomatonInput {
    pub switch: bool,
    pub i: i64,
}
pub struct AutomatonState {
    mem: State,
    mem_1: i64,
    sum: SumState,
}
impl AutomatonState {
    pub fn init() -> AutomatonState {
        AutomatonState {
            mem: State::Off,
            mem_1: 0i64,
            sum: SumState::init(),
        }
    }
    pub fn step(&mut self, input: AutomatonInput) -> i64 {
        let state = self.mem;
        let (next_state, x, o) = match state {
            State::Off => {
                let next_state = if input.switch { State::On } else { state };
                let x = self.mem_1;
                let o = 0i64;
                (next_state, x, o)
            }
            State::On => {
                let next_state = if input.switch { State::Off } else { state };
                let x = self.sum.step(SumInput {
                    reset: input.switch,
                    i: input.i,
                });
                let o = 10i64 * x;
                (next_state, x, o)
            }
        };
        self.mem = next_state;
        self.mem_1 = x;
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
