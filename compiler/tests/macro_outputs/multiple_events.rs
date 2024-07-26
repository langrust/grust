pub struct MultipleEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct MultipleEventsState {
    mem: i64,
}
impl MultipleEventsState {
    pub fn init() -> MultipleEventsState {
        MultipleEventsState { mem: 0i64 }
    }
    pub fn step(&mut self, input: MultipleEventsInput) -> (i64, f64) {
        let z = match (input.a, input.b) {
            (Some(a), Some(b)) if input.v > 50i64 => {
                let z = 1i64;
                z
            }
            (Some(a), _) => {
                let z = 2i64;
                z
            }
            (_, Some(b)) => {
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                z
            }
            (_, _) => {
                let z = self.mem;
                z
            }
        };
        let c = z;
        let d = match (input.a, input.b) {
            (Some(_), Some(_)) => 0.1,
            _ => 0.2,
        };
        self.mem = c;
        (c, d)
    }
}
