pub struct DefineEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct DefineEventsState {
    mem: i64,
}
impl DefineEventsState {
    pub fn init() -> DefineEventsState {
        DefineEventsState {
            mem: default::Default(),
        }
    }
    pub fn step(&mut self, input: DefineEventsInput) -> (i64, Option<f64>, Option<i64>) {
        let (z, y, x) = match (input.a, input.b) {
            (Some(a), Some(e)) => {
                let y = Some(());
                let z = if input.v > 50i64 { e } else { a };
                (z, y, None)
            }
            (Some(_), _) => {
                let x = Some(2i64);
                let z = 2i64;
                (z, None, x)
            }
            (_, Some(_)) => {
                let x = Some(2i64);
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                (z, None, x)
            }
            (_, _) => (self.mem, None, None),
        };
        let c = z;
        let d = match (y) {
            (Some(_)) => Some(0.1),
            _ => None,
        };
        self.mem = z;
        (c, d, x)
    }
}
