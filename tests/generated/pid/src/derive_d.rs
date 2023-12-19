pub struct DeriveDInput {
    pub x: f64,
    pub dt: f64,
}
pub struct DeriveDState {
    mem_prev_x: f64,
}
impl DeriveDState {
    pub fn init() -> DeriveDState {
        DeriveDState { mem_prev_x: 0f64 }
    }
    pub fn step(&mut self, input: DeriveDInput) -> f64 {
        let prev_x = self.mem_prev_x;
        let d = (input.x - prev_x) / input.dt;
        self.mem_prev_x = input.x;
        d
    }
}
