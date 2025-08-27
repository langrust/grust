pub struct NextInput {
    pub i: i64,
}
pub struct NextOutput {
    pub next_o: i64,
}
pub struct NextState {
    last_i: i64,
}
impl grust::core::Component for NextState {
    type Input = NextInput;
    type Output = NextOutput;
    fn init() -> NextState {
        NextState { last_i: 1i64 }
    }
    fn step(&mut self, input: NextInput) -> NextOutput {
        let next_o = input.i + self.last_i;
        self.last_i = input.i;
        NextOutput { next_o }
    }
}
pub struct SemiFibInput {
    pub i: i64,
}
pub struct SemiFibOutput {
    pub o: i64,
}
pub struct SemiFibState {
    last_next_o: i64,
    next: NextState,
}
impl grust::core::Component for SemiFibState {
    type Input = SemiFibInput;
    type Output = SemiFibOutput;
    fn init() -> SemiFibState {
        SemiFibState {
            last_next_o: 0i64,
            next: <NextState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: SemiFibInput) -> SemiFibOutput {
        let next_o = {
            let NextOutput { next_o } = <NextState as grust::core::Component>::step(
                &mut self.next,
                NextInput { i: input.i },
            );
            (next_o)
        };
        let o = self.last_next_o;
        self.last_next_o = next_o;
        SemiFibOutput { o }
    }
}
pub struct FibCallInput {}
pub struct FibCallOutput {
    pub fib: i64,
}
pub struct FibCallState {
    last_next_o_1: i64,
    next: NextState,
    next_1: NextState,
}
impl grust::core::Component for FibCallState {
    type Input = FibCallInput;
    type Output = FibCallOutput;
    fn init() -> FibCallState {
        FibCallState {
            last_next_o_1: 0i64,
            next: <NextState as grust::core::Component>::init(),
            next_1: <NextState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: FibCallInput) -> FibCallOutput {
        let fib = self.last_next_o_1;
        let next_o = {
            let NextOutput { next_o } =
                <NextState as grust::core::Component>::step(&mut self.next, NextInput { i: fib });
            (next_o)
        };
        let next_o_1 = {
            let NextOutput { next_o } = <NextState as grust::core::Component>::step(
                &mut self.next_1,
                NextInput { i: next_o },
            );
            (next_o)
        };
        self.last_next_o_1 = next_o_1;
        FibCallOutput { fib }
    }
}
pub struct FibInput {}
pub struct FibOutput {
    pub fib: i64,
}
pub struct FibState {
    last_fib: i64,
    last_next_o: i64,
}
impl grust::core::Component for FibState {
    type Input = FibInput;
    type Output = FibOutput;
    fn init() -> FibState {
        FibState {
            last_fib: 1i64,
            last_next_o: 0i64,
        }
    }
    fn step(&mut self, input: FibInput) -> FibOutput {
        let fib = self.last_next_o;
        let next_o = fib + self.last_fib;
        self.last_fib = fib;
        self.last_next_o = next_o;
        FibOutput { fib }
    }
}
