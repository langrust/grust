pub struct TestThreadsAuxInput {
    pub i: i64,
}
pub struct TestThreadsAuxState {
    last_i: i64,
}
impl TestThreadsAuxState {
    pub fn init() -> TestThreadsAuxState {
        TestThreadsAuxState { last_i: 0i64 }
    }
    pub fn step(&mut self, input: TestThreadsAuxInput) -> i64 {
        let (i1, i3, i2) = {
            let (i1, i3, i2) = ({ (input.i - 54i64) * 2i64 }, { 7i64 * input.i }, {
                (input.i + 54i64) * 2i64
            });
            (i1, i3, i2)
        };
        let (i12, i23) = {
            let (i12, i23) = ({ i1 + i2 }, { i2 + i3 });
            (i12, i23)
        };
        let i123 = (i12 + (2i64 * i3)) + i23;
        let next_o = match input.i {
            0 => {
                let next_o = 1i64 + self.last_i;
                next_o
            }
            7 => {
                let next_o = i123;
                next_o
            }
            _ => {
                let next_o = i12;
                next_o
            }
        };
        self.last_i = input.i;
        next_o
    }
}
pub struct TestThreadsInput {
    pub i: i64,
}
pub struct TestThreadsState {
    last_i: i64,
    test_threads_aux: TestThreadsAuxState,
    test_threads_aux_1: TestThreadsAuxState,
    test_threads_aux_2: TestThreadsAuxState,
}
impl TestThreadsState {
    pub fn init() -> TestThreadsState {
        TestThreadsState {
            last_i: 0i64,
            test_threads_aux: TestThreadsAuxState::init(),
            test_threads_aux_1: TestThreadsAuxState::init(),
            test_threads_aux_2: TestThreadsAuxState::init(),
        }
    }
    pub fn step(&mut self, input: TestThreadsInput) -> i64 {
        let ((i1_1, i1_2), i1_3) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope.spawn(|| {
                self.test_threads_aux
                    .step(TestThreadsAuxInput { i: input.i })
            });
            let (i1_1, i1_2) = ({ (input.i - 54i64) * 2i64 }, { (input.i + 54i64) * 2i64 });
            let i1_3 = (reserved_grust_thread_kid_0
                .join()
                .expect("unexpected panic in sub-thread"));
            ((i1_1, i1_2), i1_3)
        });
        let ((x, i2_1), (x_1, i2_2)) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope.spawn(|| {
                let x = (i1_1 + i1_2) - i1_3;
                let i2_1 = self.test_threads_aux_1.step(TestThreadsAuxInput { i: x });
                (x, i2_1)
            });
            let reserved_grust_thread_kid_1 = reserved_grust_thread_scope.spawn(|| {
                let x_1 = (i1_2 - i1_2) + i1_3;
                let i2_2 = self.test_threads_aux_2.step(TestThreadsAuxInput { i: x_1 });
                (x_1, i2_2)
            });
            let ((x, i2_1), (x_1, i2_2)) = (
                reserved_grust_thread_kid_0
                    .join()
                    .expect("unexpected panic in sub-thread"),
                reserved_grust_thread_kid_1
                    .join()
                    .expect("unexpected panic in sub-thread"),
            );
            ((x, i2_1), (x_1, i2_2))
        });
        let next_o = match input.i {
            0 => {
                let next_o = 1i64 + self.last_i;
                next_o
            }
            7 => {
                let next_o = i2_1;
                next_o
            }
            _ => {
                let next_o = i2_2;
                next_o
            }
        };
        self.last_i = input.i;
        next_o
    }
}
