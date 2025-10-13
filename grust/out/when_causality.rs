pub struct MatchOkInput {
    pub input: i64,
}
pub struct MatchOkOutput {
    pub sampled: i64,
}
pub struct MatchOkState {}
impl grust::core::Component for MatchOkState {
    type Input = MatchOkInput;
    type Output = MatchOkOutput;
    fn init() -> MatchOkState {
        MatchOkState {}
    }
    fn step(&mut self, input: MatchOkInput) -> MatchOkOutput {
        let (sampled, mem) = match input.input {
            x if x < 0i64 => {
                let sampled = input.input;
                let mem = sampled;
                (sampled, mem)
            }
            _ => {
                let mem = input.input;
                let sampled = mem;
                (sampled, mem)
            }
        };
        MatchOkOutput { sampled }
    }
}
pub struct WhenNowOkInput {
    pub input: Option<i64>,
    pub ck: Option<f64>,
}
pub struct WhenNowOkOutput {
    pub sampled: Option<i64>,
}
pub struct WhenNowOkState {
    last_mem: i64,
}
impl grust::core::Component for WhenNowOkState {
    type Input = WhenNowOkInput;
    type Output = WhenNowOkOutput;
    fn init() -> WhenNowOkState {
        WhenNowOkState { last_mem: 0i64 }
    }
    fn step(&mut self, input: WhenNowOkInput) -> WhenNowOkOutput {
        let (sampled, mem) = match (input.input, input.ck) {
            (Some(input), _) => {
                let mem = input;
                (None, mem)
            }
            (_, Some(ck)) => {
                let mem = self.last_mem;
                let sampled = Some(mem);
                (sampled, mem)
            }
            (_, _) => {
                let mem = self.last_mem;
                (None, mem)
            }
        };
        self.last_mem = mem;
        WhenNowOkOutput { sampled }
    }
}
pub struct WhenOkInput {
    pub input: Option<i64>,
    pub ck: Option<f64>,
}
pub struct WhenOkOutput {
    pub sampled: Option<i64>,
}
pub struct WhenOkState {
    last_mem: i64,
}
impl grust::core::Component for WhenOkState {
    type Input = WhenOkInput;
    type Output = WhenOkOutput;
    fn init() -> WhenOkState {
        WhenOkState { last_mem: 0i64 }
    }
    fn step(&mut self, input: WhenOkInput) -> WhenOkOutput {
        let mem = match (input.input) {
            (Some(input)) => input,
            (_) => {
                let mem = self.last_mem;
                mem
            }
        };
        let sampled = match (input.ck) {
            (Some(ck)) => Some(mem),
            (_) => None,
        };
        self.last_mem = mem;
        WhenOkOutput { sampled }
    }
}
