pub struct Test1Input {
    pub i: i64,
}
pub struct Test1State {
    last_i: i64,
}
impl Test1State {
    pub fn init() -> Test1State {
        Test1State { last_i: 0i64 }
    }
    pub fn step(&mut self, input: Test1Input) -> i64 {
        let i1 = (input.i - 54i64) * 2i64;
        let i2 = (input.i + 54i64) * 2i64;
        let i3 = 7i64 * input.i;
        let i12 = i1 + i2;
        let i23 = i2 + i3;
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
