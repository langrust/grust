pub struct MultipleEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct MultipleEventsState {}
impl MultipleEventsState {
    pub fn init() -> MultipleEventsState {
        MultipleEventsState {}
    }
    pub fn step(&mut self, input: MultipleEventsInput) -> i64 {
        let z = match (input.a, input.b) {
            (Some(a), Some(b)) => {
                let z = if input.v > 50i64 { a } else { b };
                z
            }
            (Some(a), _) => {
                let z = a;
                z
            }
            (_, Some(b)) => {
                let z = if input.v > 50i64 { 0i64 } else { b };
                z
            }
            (_, _) => {
                let z = 5i64;
                z
            }
        };
        let c = z;
        c
    }
}
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
        DefineEventsState { mem: 0i64 }
    }
    pub fn step(&mut self, input: DefineEventsInput) -> (i64, f64, Option<i64>) {
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
            (_, _) => {
                let z = self.mem;
                (z, None, None)
            }
        };
        let c = z;
        let d = match (y) {
            (Some(a)) => 0.1,
            _ => 0.2,
        };
        self.mem = c;
        (c, d, x)
    }
}
