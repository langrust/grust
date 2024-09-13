pub struct RisingEdgesInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct RisingEdgesState {
    mem: i64,
    mem_1: f64,
    mem_2: bool,
    mem_3: bool,
    mem_4: bool,
    mem_5: i64,
    mem_6: i64,
    mem_7: bool,
    mem_8: i64,
}
impl RisingEdgesState {
    pub fn init() -> RisingEdgesState {
        RisingEdgesState {
            mem: Default::default(),
            mem_1: Default::default(),
            mem_2: false,
            mem_3: false,
            mem_4: false,
            mem_5: Default::default(),
            mem_6: Default::default(),
            mem_7: false,
            mem_8: Default::default(),
        }
    }
    pub fn step(&mut self, input: RisingEdgesInput) -> (i64, f64, Option<i64>) {
        let c = match (input.a) {
            (Some(a)) => a,
            _ => self.mem,
        };
        let (z, y, x) = match (input.a, input.b) {
            (Some(a), Some(e)) if x_1 && !self.mem_2 => {
                let y = Some(());
                let z = if input.v > 80i64 { e } else { a };
                (z, y, None)
            }
            (Some(a), _) if (a != 0i64) && (x_2 && !self.mem_3) => {
                let x = Some(2i64);
                let z = 2i64;
                (z, None, x)
            }
            (_, Some(e)) => {
                let x = match () {
                    () if x_4 && !self.mem_7 => Some(2i64),
                    _ => None,
                };
                let z = match () {
                    () if x_3 && !self.mem_4 => input.v + self.mem_5,
                    _ => self.mem_6,
                };
                let x_3 = input.v > 50i64;
                let x_4 = e < 20i64;
                (z, None, x)
            }
            (_, _) => (self.mem_8, None, None),
        };
        let d = match (y) {
            (Some(_)) => 0.1,
            _ => self.mem_1,
        };
        let x_1 = input.v > 50i64;
        let x_2 = input.v < 40i64;
        self.mem = c;
        self.mem_1 = d;
        self.mem_2 = x_1;
        self.mem_3 = x_2;
        self.mem_4 = x_3;
        self.mem_5 = c;
        self.mem_6 = z;
        self.mem_7 = x_4;
        self.mem_8 = z;
        (c, d, x)
    }
}
