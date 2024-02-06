use creusot_contracts::{ensures, requires};

pub struct TestOInput {
    pub i1: i64,
    pub i2: i64,
}
pub struct TestOState {
    pub mem_z: i64,
}
impl TestOState {
    #[ensures(1000i64 >= result.mem_z)]
    #[ensures(result.mem_z > 0i64)]
    pub fn init() -> TestOState {
        TestOState { mem_z: 1i64 }
    }
    #[requires(1000i64 > input.i1)]
    #[requires(input.i1 > 0i64)]
    #[requires(1000i64 >= input.i2)]
    #[requires(input.i2 > input.i1)]
    #[requires(1000i64 >= self.mem_z)]
    #[requires(self.mem_z > 0i64)]
    #[ensures(1000i64 >= (^self).mem_z)]
    #[ensures((^self).mem_z > 0i64)]
    #[ensures(input.i1 >= result)]
    #[ensures(result >= 0i64)]
    pub fn step(&mut self, input: TestOInput) -> i64 {
        let x = input.i2 - input.i1;
        let y = input.i1 / x;
        let z = self.mem_z;
        let o = y / z;
        let z_prime = z + x;
        self.mem_z = if z_prime > 1000i64 { 1i64 } else { z_prime };
        
        o
    }
}
