pub struct TestMixedAuxInput {
    pub i: i64,
}
pub struct TestMixedAuxOutput {
    pub next_o: i64,
}
pub struct TestMixedAuxState {
    last_i: i64,
}
impl grust::core::Component for TestMixedAuxState {
    type Input = TestMixedAuxInput;
    type Output = TestMixedAuxOutput;
    fn init() -> TestMixedAuxState {
        TestMixedAuxState { last_i: 0i64 }
    }
    fn step(&mut self, input: TestMixedAuxInput) -> TestMixedAuxOutput {
        let ((i3, i1, i2), ()) = {
            let (i3, i1, i2) = ({ 7i64 * input.i }, { (input.i - 54i64) * 2i64 }, {
                (input.i + 54i64) * 2i64
            });
            ((i3, i1, i2), ())
        };
        let ((i12, i23), ()) = {
            let (i12, i23) = ({ i1 + i2 }, { i2 + i3 });
            ((i12, i23), ())
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
        TestMixedAuxOutput { next_o }
    }
}
pub struct TestMixedInput {
    pub i: i64,
}
pub struct TestMixedOutput {
    pub next_o: i64,
}
pub struct TestMixedState {
    last_i: i64,
    test_mixed_aux: TestMixedAuxState,
    test_mixed_aux_1: TestMixedAuxState,
    test_mixed_aux_2: TestMixedAuxState,
}
impl grust::core::Component for TestMixedState {
    type Input = TestMixedInput;
    type Output = TestMixedOutput;
    fn init() -> TestMixedState {
        TestMixedState {
            last_i: 0i64,
            test_mixed_aux: <TestMixedAuxState as grust::core::Component>::init(),
            test_mixed_aux_1: <TestMixedAuxState as grust::core::Component>::init(),
            test_mixed_aux_2: <TestMixedAuxState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: TestMixedInput) -> TestMixedOutput {
        let ((i1_1, i1_2), i1_3) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope.spawn(|| {
                let TestMixedAuxOutput { next_o } =
                    <TestMixedAuxState as grust::core::Component>::step(
                        &mut self.test_mixed_aux,
                        TestMixedAuxInput { i: input.i },
                    );
                (next_o)
            });
            let (i1_1, i1_2) = ({ (input.i - 54i64) * 2i64 }, { (input.i + 54i64) * 2i64 });
            let i1_3 = {
                reserved_grust_thread_kid_0
                    .join()
                    .expect("unexpected panic in sub-thread")
            };
            ((i1_1, i1_2), i1_3)
        });
        let ((x, i2_1), (x_1, i2_2)) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope.spawn(|| {
                let x = (i1_1 + i1_2) - i1_3;
                let i2_1 = {
                    let TestMixedAuxOutput { next_o } =
                        <TestMixedAuxState as grust::core::Component>::step(
                            &mut self.test_mixed_aux_1,
                            TestMixedAuxInput { i: x },
                        );
                    (next_o)
                };
                (x, i2_1)
            });
            let reserved_grust_thread_kid_1 = reserved_grust_thread_scope.spawn(|| {
                let x_1 = (i1_2 - i1_2) + i1_3;
                let i2_2 = {
                    let TestMixedAuxOutput { next_o } =
                        <TestMixedAuxState as grust::core::Component>::step(
                            &mut self.test_mixed_aux_2,
                            TestMixedAuxInput { i: x_1 },
                        );
                    (next_o)
                };
                (x_1, i2_2)
            });
            let ((x, i2_1), (x_1, i2_2)) = (
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
        TestMixedOutput { next_o }
    }
}
