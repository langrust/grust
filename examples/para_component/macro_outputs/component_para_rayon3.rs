pub struct TestRayon3AuxInput {
    pub i: i64,
}
pub struct TestRayon3AuxState {
    last_i: i64,
}
impl TestRayon3AuxState {
    pub fn init() -> TestRayon3AuxState {
        TestRayon3AuxState { last_i: 0i64 }
    }
    pub fn step(&mut self, input: TestRayon3AuxInput) -> i64 {
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
pub struct TestRayon3Input {
    pub i: i64,
}
pub struct TestRayon3State {
    last_i: i64,
    test_rayon3_aux: TestRayon3AuxState,
    test_rayon3_aux_1: TestRayon3AuxState,
    test_rayon3_aux_2: TestRayon3AuxState,
}
impl TestRayon3State {
    pub fn init() -> TestRayon3State {
        TestRayon3State {
            last_i: 0i64,
            test_rayon3_aux: TestRayon3AuxState::init(),
            test_rayon3_aux_1: TestRayon3AuxState::init(),
            test_rayon3_aux_2: TestRayon3AuxState::init(),
        }
    }
    pub fn step(&mut self, input: TestRayon3Input) -> i64 {
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
                        self.test_rayon3_aux.step(TestRayon3AuxInput { i: input.i })
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
                        let i2_1 = self.test_rayon3_aux_1.step(TestRayon3AuxInput { i: x });
                        (x, i2_1)
                    }
                },
                {
                    {
                        let x_1 = (i1_2 - i1_2) + i1_3;
                        let i2_2 = self.test_rayon3_aux_2.step(TestRayon3AuxInput { i: x_1 });
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
