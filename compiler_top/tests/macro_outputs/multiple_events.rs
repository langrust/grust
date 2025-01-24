pub struct MultipleEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct MultipleEventsState {
    last_a_bis: i64,
    last_d: i64,
    last_z: i64,
}
impl MultipleEventsState {
    pub fn init() -> MultipleEventsState {
        MultipleEventsState {
            last_a_bis: 0i64,
            last_d: 0i64,
            last_z: 0i64,
        }
    }
    pub fn step(&mut self, input: MultipleEventsInput) -> (i64, i64) {
        let (a_bis, z) = match (input.a, input.b) {
            (Some(a), Some(b)) if input.v > 50i64 => {
                let z = self.last_a_bis;
                let a_bis = a;
                (a_bis, z)
            }
            (Some(a), _) => {
                let z = 2i64;
                let a_bis = a;
                (a_bis, z)
            }
            (_, Some(b)) => {
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                (self.last_a_bis, z)
            }
            (_, _) => (self.last_a_bis, self.last_z),
        };
        let c = z;
        let d = match (input.a, input.b) {
            (Some(a), Some(b)) => (10i64 * a) + b,
            (_, _) => self.last_d,
        };
        self.last_a_bis = a_bis;
        self.last_d = d;
        self.last_z = z;
        (c, d)
    }
}
