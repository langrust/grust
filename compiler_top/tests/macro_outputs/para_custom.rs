pub struct NextInput {
    pub i: i64,
}
pub struct NextState {
    last_i: i64,
}
impl grust::core::Component for NextState {
    type Input = NextInput;
    type Output = i64;
    fn init() -> NextState {
        NextState { last_i: 1i64 }
    }
    fn step(&mut self, input: NextInput) -> i64 {
        let next_o = module::add_i64(input.i, self.last_i);
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
impl grust::core::Component for SemiFibState {
    type Input = SemiFibInput;
    type Output = i64;
    fn init() -> SemiFibState {
        SemiFibState {
            last_next_o: 0i64,
            next: <NextState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: SemiFibInput) -> i64 {
        let (o, next_o) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope.spawn(|| {
                <NextState as grust::core::Component>::step(
                    &mut self.next,
                    NextInput { i: input.i },
                )
            });
            let o = { self.last_next_o };
            let next_o = {
                reserved_grust_thread_kid_0
                    .join()
                    .expect("unexpected panic in sub-thread")
            };
            (o, next_o)
        });
        self.last_next_o = next_o;
        o
    }
}
pub struct FibCallInput {}
pub struct FibCallState {
    last_next_o_1: i64,
    next: NextState,
    next_1: NextState,
}
impl grust::core::Component for FibCallState {
    type Input = FibCallInput;
    type Output = i64;
    fn init() -> FibCallState {
        FibCallState {
            last_next_o_1: 0i64,
            next: <NextState as grust::core::Component>::init(),
            next_1: <NextState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: FibCallInput) -> i64 {
        let fib = self.last_next_o_1;
        let (next_o, next_o_1) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope.spawn(|| {
                <NextState as grust::core::Component>::step(&mut self.next, NextInput { i: fib })
            });
            let reserved_grust_thread_kid_1 = reserved_grust_thread_scope.spawn(|| {
                <NextState as grust::core::Component>::step(&mut self.next_1, NextInput { i: fib })
            });
            let (next_o, next_o_1) = (
                {
                    reserved_grust_thread_kid_0
                        .join()
                        .expect("unexpected panic in sub-thread")
                },
                {
                    reserved_grust_thread_kid_1
                        .join()
                        .expect("unexpected panic in sub-thread")
                },
            );
            (next_o, next_o_1)
        });
        self.last_next_o_1 = next_o_1;
        fib
    }
}
pub struct FibInput {}
pub struct FibState {
    last_fib: i64,
    last_next_o: i64,
}
impl grust::core::Component for FibState {
    type Input = FibInput;
    type Output = i64;
    fn init() -> FibState {
        FibState {
            last_fib: 1i64,
            last_next_o: 0i64,
        }
    }
    fn step(&mut self, input: FibInput) -> i64 {
        let fib = self.last_next_o;
        let next_o = fib + self.last_fib;
        self.last_fib = fib;
        self.last_next_o = next_o;
        fib
    }
}
