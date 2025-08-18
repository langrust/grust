pub struct DefineEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct DefineEventsOutput {
    pub c: i64,
    pub d: Option<f64>,
    pub x: Option<i64>,
}
pub struct DefineEventsState {
    last_z: i64,
}
impl grust::core::Component for DefineEventsState {
    type Input = DefineEventsInput;
    type Output = DefineEventsOutput;
    fn init() -> DefineEventsState {
        DefineEventsState { last_z: 0i64 }
    }
    fn step(&mut self, input: DefineEventsInput) -> DefineEventsOutput {
        let (z, y, x) = match (input.a, input.b) {
            (Some(a), Some(e)) => {
                let y = Some(());
                let z = if input.v > 50i64 { e } else { a };
                (z, y, None)
            }
            (Some(_), _) => {
                let x = Some(2i64);
                (self.last_z, None, x)
            }
            (_, Some(_)) => {
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                let x = Some(2i64);
                (z, None, x)
            }
            (_, _) => (self.last_z, None, None),
        };
        let c = z;
        let d = match (y) {
            (Some(y)) => Some(0.1f64),
            (_) => None,
        };
        self.last_z = z;
        DefineEventsOutput { c, d, x }
    }
}
