pub struct Test1AuxInput {
    pub i: i64,
}
pub struct Test1AuxState {
    last_i: i64,
}
impl Test1AuxState {
    pub fn init() -> Test1AuxState {
        Test1AuxState { last_i: 0i64 }
    }
    pub fn step(&mut self, input: Test1AuxInput) -> i64 {
        let (i1, i3, i2) = {
            let (i1, i3, i2) = (
                (input.i - 54i64) * 2i64,
                7i64 * input.i,
                (input.i + 54i64) * 2i64,
            );
            (i1, i3, i2)
        };
        let (i12, i23) = {
            let (i12, i23) = (i1 + i2, i2 + i3);
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
pub struct Test1Input {
    pub i: i64,
}
pub struct Test1State {
    last_i: i64,
    test1_aux: Test1AuxState,
    test1_aux_1: Test1AuxState,
    test1_aux_2: Test1AuxState,
}
impl Test1State {
    pub fn init() -> Test1State {
        Test1State {
            last_i: 0i64,
            test1_aux: Test1AuxState::init(),
            test1_aux_1: Test1AuxState::init(),
            test1_aux_2: Test1AuxState::init(),
        }
    }
    pub fn step(&mut self, input: Test1Input) -> i64 {
        let ((i1_1, i1_2), i1_3) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope
                .spawn(|| self.test1_aux.step(Test1AuxInput { i: input.i }));
            let (i1_1, i1_2) = ((input.i - 54i64) * 2i64, (input.i + 54i64) * 2i64);
            let i1_3 = (reserved_grust_thread_kid_0
                .join()
                .expect("unexpected panic in sub-thread"));
            ((i1_1, i1_2), i1_3)
        });
        let ((x, i2_1), (x_1, i2_2)) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope.spawn(|| {
                let x = (i1_1 + i1_2) - i1_3;
                let i2_1 = self.test1_aux_1.step(Test1AuxInput { i: x });
                (x, i2_1)
            });
            let reserved_grust_thread_kid_1 = reserved_grust_thread_scope.spawn(|| {
                let x_1 = (i1_2 - i1_2) + i1_3;
                let i2_2 = self.test1_aux_2.step(Test1AuxInput { i: x_1 });
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
