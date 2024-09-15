pub struct RisingEdgesInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct RisingEdgesState {
    last_c: i64,
    last_d: f64,
    last_test: bool,
    last_x_1: bool,
    last_x_2: bool,
    last_x_3: bool,
    last_x_4: bool,
    last_x_5: bool,
    last_z: i64,
}
impl RisingEdgesState {
    pub fn init() -> RisingEdgesState {
        RisingEdgesState {
            last_c: Default::default(),
            last_d: Default::default(),
            last_test: false,
            last_x_1: false,
            last_x_2: false,
            last_x_3: false,
            last_x_4: false,
            last_x_5: false,
            last_z: Default::default(),
        }
    }
    pub fn step(&mut self, input: RisingEdgesInput) -> (i64, f64, Option<i64>) {
        let c = match (input.a) {
            (Some(a)) => a,
            _ => self.last_c,
        };
        let x_4 = input.v < 40i64;
        let x_3 = input.v > 50i64;
        let (z, y, x) = match (input.a, input.b) {
            (Some(a), Some(e)) if x_3 && !self.last_x_3 => {
                let y = Some(());
                let z = if input.v > 80i64 { e } else { a };
                (z, y, None)
            }
            (Some(a), _) if (a != 0i64) && (x_4 && !self.last_x_4) => {
                let x = Some(2i64);
                let z = 2i64;
                (z, None, x)
            }
            (_, Some(e)) => {
                let x_5 = e < 20i64;
                let x = match () {
                    () if x_5 && !self.last_x_5 => Some(2i64),
                    _ => None,
                };
                (self.last_z, None, x)
            }
            (_, _) => (self.last_z, None, None),
        };
        let d = match (y) {
            (Some(_)) => 0.1,
            _ => self.last_d,
        };
        let test = input.v > 50i64;
        let w2 = match () {
            () if test && !self.last_test => Some(test),
            _ => None,
        };
        let x_1 = input.v > 50i64;
        let w3 = match () {
            () if x_1 && !self.last_x_1 => Some(false),
            _ => None,
        };
        let x_2 = input.v > 50i64;
        let w = match () {
            () if x_2 && !self.last_x_2 => Some(input.v + self.last_c),
            _ => None,
        };
        self.last_c = c;
        self.last_d = d;
        self.last_test = test;
        self.last_x_1 = x_1;
        self.last_x_2 = x_2;
        self.last_x_3 = x_3;
        self.last_x_4 = x_4;
        self.last_x_5 = x_5;
        self.last_z = z;
        (c, d, x)
    }
}
