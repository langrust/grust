pub struct DefineEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct DefineEventsState {
    last_z: i64,
}
impl DefineEventsState {
    pub fn init() -> DefineEventsState {
        DefineEventsState {
            last_z: Default::default(),
        }
    }
    pub fn step(&mut self, input: DefineEventsInput) -> (i64, Option<f64>, Option<i64>) {
        let (y, z, x) = match (input.a, input.b) {
            (Some(a), Some(e)) => {
                let z = if input.v > 50i64 { e } else { a };
                let y = Some(());
                (y, z, None)
            }
            (Some(_), _) => {
                let x = Some(2i64);
                let z = 2i64;
                (None, z, x)
            }
            (_, Some(_)) => {
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                let x = Some(2i64);
                (None, z, x)
            }
            (_, _) => (None, self.last_z, None),
        };
        let c = z;
        let d = match (y) {
            (Some(_)) => Some(0.1f64),
            _ => None,
        };
        self.last_z = z;
        (c, d, x)
    }
}
