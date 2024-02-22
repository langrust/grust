use creusot_contracts::{ensures, requires};

pub struct TestOInput {
    pub i1: i64,
    pub i2: i64,
}
pub struct TestOState {
    mem_z: i64,
}
impl TestOState {
    pub fn init() -> TestOState {
        TestOState { mem_z: 1i64 }
    }
    #[requires(0i64<input.i1||input.i1<input.i2)]
    #[ensures(result>= 0i64)]
    #[requires(self.mem_z>0i64)]
    #[ensures((^self).mem_z>0i64)]
    pub fn step(&mut self, input: TestOInput) -> i64 {
        let z = self.mem_z;
        let x = input.i2 - input.i1;
        let y = input.i1 / x;
        let o = y / z;
        let z_prime = z + x;
        self.mem_z = if z_prime > 1000i64 { 1i64 } else { z_prime };
        o
    }
}
