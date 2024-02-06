use crate::functions::min;
pub struct IntegrateIInput {
    pub x: f64,
    pub dt: f64,
}
pub struct IntegrateIState {
    mem_prev_i: f64,
}
impl IntegrateIState {
    pub fn init() -> IntegrateIState {
        IntegrateIState {
            mem_prev_i: 0f64,
        }
    }
    pub fn step(&mut self, input: IntegrateIInput) -> f64 {
        let MAX_INTEGRALE = 1000f64;
        let prev_i = self.mem_prev_i;
        let i = Expr::FunctionCall(
            parse_quote! {
                min(prev_i + input.x * input.dt, MAX_INTEGRALE)
            },
        );
        self.mem_prev_i = i;
        i
    }
}
