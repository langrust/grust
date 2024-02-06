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
