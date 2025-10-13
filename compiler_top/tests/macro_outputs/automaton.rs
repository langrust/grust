#[derive(Clone, Copy, PartialEq, Default, Debug)]
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
pub struct SumOutput {
    pub o: i64,
}
pub struct SumState {
    last_o: i64,
}
impl grust::core::Component for SumState {
    type Input = SumInput;
    type Output = SumOutput;
    fn init() -> SumState {
        SumState { last_o: 0i64 }
    }
    fn step(&mut self, input: SumInput) -> SumOutput {
        let x = add(self.last_o, input.i);
        let o = if input.reset { 0i64 } else { x };
        self.last_o = o;
        SumOutput { o }
    }
}
pub struct AutomatonInput {
    pub switch: bool,
    pub i: i64,
}
pub struct AutomatonOutput {
    pub o: i64,
}
pub struct AutomatonState {
    last_next_state: State,
    last_x: i64,
    sum: SumState,
}
impl grust::core::Component for AutomatonState {
    type Input = AutomatonInput;
    type Output = AutomatonOutput;
    fn init() -> AutomatonState {
        AutomatonState {
            last_next_state: State::Off,
            last_x: 0i64,
            sum: <SumState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: AutomatonInput) -> AutomatonOutput {
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
                let x = {
                    let SumOutput { o } = <SumState as grust::core::Component>::step(
                        &mut self.sum,
                        SumInput {
                            reset: input.switch,
                            i: input.i,
                        },
                    );
                    (o)
                };
                let o = 10i64 * x;
                (next_state, x, o)
            }
        };
        self.last_next_state = next_state;
        self.last_x = x;
        AutomatonOutput { o }
    }
}
