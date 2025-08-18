pub struct TestRayon1AuxInput {
    pub i: i64,
}
pub struct TestRayon1AuxOutput {
    pub next_o: i64,
}
pub struct TestRayon1AuxState {
    last_i: i64,
}
impl grust::core::Component for TestRayon1AuxState {
    type Input = TestRayon1AuxInput;
    type Output = TestRayon1AuxOutput;
    fn init() -> TestRayon1AuxState {
        TestRayon1AuxState { last_i: 0i64 }
    }
    fn step(&mut self, input: TestRayon1AuxInput) -> TestRayon1AuxOutput {
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
        TestRayon1AuxOutput { next_o }
    }
}
pub struct TestRayon1Input {
    pub i: i64,
}
pub struct TestRayon1Output {
    pub next_o: i64,
}
pub struct TestRayon1State {
    last_i: i64,
    test_rayon1_aux: TestRayon1AuxState,
    test_rayon1_aux_1: TestRayon1AuxState,
    test_rayon1_aux_2: TestRayon1AuxState,
}
impl grust::core::Component for TestRayon1State {
    type Input = TestRayon1Input;
    type Output = TestRayon1Output;
    fn init() -> TestRayon1State {
        TestRayon1State {
            last_i: 0i64,
            test_rayon1_aux: <TestRayon1AuxState as grust::core::Component>::init(),
            test_rayon1_aux_1: <TestRayon1AuxState as grust::core::Component>::init(),
            test_rayon1_aux_2: <TestRayon1AuxState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: TestRayon1Input) -> TestRayon1Output {
        let ((i1_1, i1_2, i1_3), ()) = {
            let (i1_1, i1_2, i1_3) = (
                { (input.i - 54i64) * 2i64 },
                { (input.i + 54i64) * 2i64 },
                {
                    {
                        let TestRayon1AuxOutput { next_o } =
                            <TestRayon1AuxState as grust::core::Component>::step(
                                &mut self.test_rayon1_aux,
                                TestRayon1AuxInput { i: input.i },
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
                        let TestRayon1AuxOutput { next_o } =
                            <TestRayon1AuxState as grust::core::Component>::step(
                                &mut self.test_rayon1_aux_1,
                                TestRayon1AuxInput { i: x },
                            );
                        (next_o)
                    };
                    (x, i2_1)
                },
                {
                    let x_1 = (i1_2 - i1_2) + i1_3;
                    let i2_2 = {
                        let TestRayon1AuxOutput { next_o } =
                            <TestRayon1AuxState as grust::core::Component>::step(
                                &mut self.test_rayon1_aux_2,
                                TestRayon1AuxInput { i: x_1 },
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
        TestRayon1Output { next_o }
    }
}
