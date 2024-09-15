pub struct NextInput {
    pub i: i64,
}
pub struct NextState {
    last_i: i64,
}
impl NextState {
    pub fn init() -> NextState {
        NextState { last_i: 1i64 }
    }
    pub fn step(&mut self, input: NextInput) -> i64 {
        let next_o = input.i + self.last_i;
        self.last_i = input.i;
        next_o
    }
}
pub struct SemiFibInput {
    pub i: i64,
}
pub struct SemiFibState {
    last_next_o: i64,
    next: NextState,
}
impl SemiFibState {
    pub fn init() -> SemiFibState {
        SemiFibState {
            last_next_o: Default::default(),
            next: NextState::init(),
        }
    }
    pub fn step(&mut self, input: SemiFibInput) -> i64 {
        let next_o = self.next.step(NextInput { i: input.i });
        let o = self.last_next_o;
        self.last_next_o = next_o;
        o
    }
}
pub struct FibCallInput {}
pub struct FibCallState {
    last_next_o: i64,
    next: NextState,
}
impl FibCallState {
    pub fn init() -> FibCallState {
        FibCallState {
            last_next_o: Default::default(),
            next: NextState::init(),
        }
    }
    pub fn step(&mut self, input: FibCallInput) -> i64 {
        let fib = self.last_next_o;
        let next_o = self.next.step(NextInput { i: fib });
        self.last_next_o = next_o;
        fib
    }
}
pub struct FibInput {}
pub struct FibState {
    last_fib: i64,
    last_next_o: i64,
}
impl FibState {
    pub fn init() -> FibState {
        FibState {
            last_fib: 1i64,
            last_next_o: Default::default(),
        }
    }
    pub fn step(&mut self, input: FibInput) -> i64 {
        let fib = self.last_next_o;
        let next_o = fib + self.last_fib;
        self.last_fib = fib;
        self.last_next_o = next_o;
        fib
    }
}
