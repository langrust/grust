pub struct RisingEdgesInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct RisingEdgesState {
    mem: bool,
    mem_1: bool,
    mem_2: bool,
    mem_3: bool,
    mem_4: i64,
}
impl RisingEdgesState {
    pub fn init() -> RisingEdgesState {
        RisingEdgesState {
            mem: false,
            mem_1: false,
            mem_2: false,
            mem_3: false,
            mem_4: 0i64,
        }
    }
    pub fn step(&mut self, input: RisingEdgesInput) -> (i64, f64, Option<i64>) {
        let (z, y, x) = match (input.a, input.b) {
            (Some(a), Some(e)) if input.v > 50i64 && !self.mem => {
                let y = Some(());
                let z = if input.v > 80i64 { e } else { a };
                (z, y, None)
            }
            (Some(a), _) if a != 0i64 && input.v < 40i64 && !self.mem_1 => {
                let x = Some(2i64);
                let z = 2i64;
                (z, None, x)
            }
            (_, Some(e)) => {
                let x = match () {
                    () if e < 20i64 && !self.mem_2 => Some(2i64),
                    _ => None,
                };
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                (z, None, x)
            }
            (_, _) => {
                let z = match () {
                    () if input.v > 50i64 && !self.mem_3 => input.v + self.mem_4,
                    _ => 0i64,
                };
                (z, None, None)
            }
        };
        let c = match (input.a) {
            (Some(a)) => a,
            _ => z,
        };
        let d = match (y) {
            (Some(_)) => 0.1,
            _ => 0.2,
        };
        self.mem = input.v > 50i64;
        self.mem_1 = input.v < 40i64;
        self.mem_2 = e < 20i64;
        self.mem_3 = input.v > 50i64;
        self.mem_4 = c;
        (c, d, x)
    }
}
