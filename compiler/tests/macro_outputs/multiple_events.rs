pub struct MultipleEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct MultipleEventsState {
    mem: i64,
    mem_1: i64,
}
impl MultipleEventsState {
    pub fn init() -> MultipleEventsState {
        MultipleEventsState {
            mem: Default::default(),
            mem_1: Default::default(),
        }
    }
    pub fn step(&mut self, input: MultipleEventsInput) -> (i64, i64) {
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
            (_, _) => self.mem_1,
        };
        let c = z;
        let d = match (input.a, input.b) {
            (Some(a), Some(b)) => (10i64 * a) + b,
            _ => self.mem,
        };
        self.mem = d;
        self.mem_1 = z;
        (c, d)
    }
}
