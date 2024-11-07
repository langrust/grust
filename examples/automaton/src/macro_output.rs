#[derive(Clone, Copy, PartialEq, Default)]
pub enum State {
    #[default]
    Off,
    On,
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
    last_o: i64,
}
impl SumState {
    pub fn init() -> SumState {
        SumState {
            last_o: Default::default(),
        }
    }
    pub fn step(&mut self, input: SumInput) -> i64 {
        let x = add(self.last_o, input.i);
        let o = if input.reset { 0i64 } else { x };
        self.last_o = o;
        o
    }
}
pub struct AutomatonInput {
    pub switch: bool,
    pub i: i64,
}
pub struct AutomatonState {
    last_next_state: State,
    last_x: i64,
    sum: SumState,
}
impl AutomatonState {
    pub fn init() -> AutomatonState {
        AutomatonState {
            last_next_state: Default::default(),
            last_x: Default::default(),
            sum: SumState::init(),
        }
    }
    pub fn step(&mut self, input: AutomatonInput) -> i64 {
        let state = self.last_next_state;
        let (next_state, x, o) = match state {
            State::Off => {
                let next_state = if input.switch { State::On } else { state };
                let x = self.last_x;
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
        self.last_next_state = next_state;
        self.last_x = x;
        o
    }
}
