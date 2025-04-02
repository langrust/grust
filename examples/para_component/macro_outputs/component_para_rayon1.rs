pub struct TestRayon1AuxInput {
    pub i: i64,
}
pub struct TestRayon1AuxState {
    last_i: i64,
}
impl TestRayon1AuxState {
    pub fn init() -> TestRayon1AuxState {
        TestRayon1AuxState { last_i: 0i64 }
    }
    pub fn step(&mut self, input: TestRayon1AuxInput) -> i64 {
        let ((i3, i1, i2), ()) = {
            let (i3, i1, i2) = (
                {
                    {
                        7i64 * input.i
                    }
                },
                {
                    {
                        (input.i - 54i64) * 2i64
                    }
                },
                {
                    {
                        (input.i + 54i64) * 2i64
                    }
                },
            );
            ((i3, i1, i2), ())
        };
        let ((i12, i23), ()) = {
            let (i12, i23) = (
                {
                    {
                        i1 + i2
                    }
                },
                {
                    {
                        i2 + i3
                    }
                },
            );
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
        next_o
    }
}
pub struct TestRayon1Input {
    pub i: i64,
}
pub struct TestRayon1State {
    last_i: i64,
    test_rayon1_aux: TestRayon1AuxState,
    test_rayon1_aux_1: TestRayon1AuxState,
    test_rayon1_aux_2: TestRayon1AuxState,
}
impl TestRayon1State {
    pub fn init() -> TestRayon1State {
        TestRayon1State {
            last_i: 0i64,
            test_rayon1_aux: TestRayon1AuxState::init(),
            test_rayon1_aux_1: TestRayon1AuxState::init(),
            test_rayon1_aux_2: TestRayon1AuxState::init(),
        }
    }
    pub fn step(&mut self, input: TestRayon1Input) -> i64 {
        let ((i1_1, i1_2, i1_3), ()) = {
            let (i1_1, i1_2, i1_3) = (
                {
                    {
                        (input.i - 54i64) * 2i64
                    }
                },
                {
                    {
                        (input.i + 54i64) * 2i64
                    }
                },
                {
                    {
                        self.test_rayon1_aux.step(TestRayon1AuxInput { i: input.i })
                    }
                },
            );
            ((i1_1, i1_2, i1_3), ())
        };
        let (((x, i2_1), (x_1, i2_2)), ()) = {
            let ((x, i2_1), (x_1, i2_2)) = (
                {
                    {
                        let x = (i1_1 + i1_2) - i1_3;
                        let i2_1 = self.test_rayon1_aux_1.step(TestRayon1AuxInput { i: x });
                        (x, i2_1)
                    }
                },
                {
                    {
                        let x_1 = (i1_2 - i1_2) + i1_3;
                        let i2_2 = self.test_rayon1_aux_2.step(TestRayon1AuxInput { i: x_1 });
                        (x_1, i2_2)
                    }
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
        next_o
    }
}
