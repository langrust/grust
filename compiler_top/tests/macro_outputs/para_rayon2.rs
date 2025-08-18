pub struct TestRayon2AuxInput {
    pub i: i64,
}
pub struct TestRayon2AuxOutput {
    pub next_o: i64,
}
pub struct TestRayon2AuxState {
    last_i: i64,
}
impl grust::core::Component for TestRayon2AuxState {
    type Input = TestRayon2AuxInput;
    type Output = TestRayon2AuxOutput;
    fn init() -> TestRayon2AuxState {
        TestRayon2AuxState { last_i: 0i64 }
    }
    fn step(&mut self, input: TestRayon2AuxInput) -> TestRayon2AuxOutput {
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
        TestRayon2AuxOutput { next_o }
    }
}
pub struct TestRayon2Input {
    pub i: i64,
}
pub struct TestRayon2Output {
    pub next_o: i64,
}
pub struct TestRayon2State {
    last_i: i64,
    test_rayon2_aux: TestRayon2AuxState,
    test_rayon2_aux_1: TestRayon2AuxState,
    test_rayon2_aux_2: TestRayon2AuxState,
}
impl grust::core::Component for TestRayon2State {
    type Input = TestRayon2Input;
    type Output = TestRayon2Output;
    fn init() -> TestRayon2State {
        TestRayon2State {
            last_i: 0i64,
            test_rayon2_aux: <TestRayon2AuxState as grust::core::Component>::init(),
            test_rayon2_aux_1: <TestRayon2AuxState as grust::core::Component>::init(),
            test_rayon2_aux_2: <TestRayon2AuxState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: TestRayon2Input) -> TestRayon2Output {
        let ((i1_1, i1_2, i1_3), ()) = {
            let (i1_1, i1_2, i1_3) = (
                { (input.i - 54i64) * 2i64 },
                { (input.i + 54i64) * 2i64 },
                {
                    {
                        let TestRayon2AuxOutput { next_o } =
                            <TestRayon2AuxState as grust::core::Component>::step(
                                &mut self.test_rayon2_aux,
                                TestRayon2AuxInput { i: input.i },
                            );
                        (next_o)
                    }
                },
            );
            ((i1_1, i1_2, i1_3), ())
        };
        let (((x, i2_1), (x_1, i2_2)), ()) = {
            let ((x, i2_1), (x_1, i2_2)) = (
                {
                    let x = (i1_1 + i1_2) - i1_3;
                    let i2_1 = {
                        let TestRayon2AuxOutput { next_o } =
                            <TestRayon2AuxState as grust::core::Component>::step(
                                &mut self.test_rayon2_aux_1,
                                TestRayon2AuxInput { i: x },
                            );
                        (next_o)
                    };
                    (x, i2_1)
                },
                {
                    let x_1 = (i1_2 - i1_2) + i1_3;
                    let i2_2 = {
                        let TestRayon2AuxOutput { next_o } =
                            <TestRayon2AuxState as grust::core::Component>::step(
                                &mut self.test_rayon2_aux_2,
                                TestRayon2AuxInput { i: x_1 },
                            );
                        (next_o)
                    };
                    (x_1, i2_2)
                },
            );
            (((x, i2_1), (x_1, i2_2)), ())
        };
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
        TestRayon2Output { next_o }
    }
}
