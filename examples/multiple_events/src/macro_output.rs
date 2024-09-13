pub struct MultipleEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct MultipleEventsState {
    mem: i64,
    mem_1: i64,
    mem_2: i64,
    mem_3: i64,
    mem_4: bool,
    mem_5: i64,
    mem_6: i64,
    mem_7: i64,
    mem_8: i64,
    mem_9: bool,
    mem_10: i64,
    mem_11: i64,
    mem_12: i64,
    mem_13: i64,
    mem_14: i64,
    mem_15: i64,
    mem_16: i64,
    mem_17: i64,
}
impl MultipleEventsState {
    pub fn init() -> MultipleEventsState {
        MultipleEventsState {
            mem: Default::default(),
            mem_1: Default::default(),
            mem_2: Default::default(),
            mem_3: Default::default(),
            mem_4: false,
            mem_5: Default::default(),
            mem_6: Default::default(),
            mem_7: Default::default(),
            mem_8: Default::default(),
            mem_9: false,
            mem_10: Default::default(),
            mem_11: Default::default(),
            mem_12: Default::default(),
            mem_13: Default::default(),
            mem_14: Default::default(),
            mem_15: Default::default(),
            mem_16: Default::default(),
            mem_17: Default::default(),
        }
    }
    pub fn step(&mut self, input: MultipleEventsInput) -> i64 {
        let c = self.mem;
        let (aux2, z, aux3, aux1) = match (input.a, input.b) {
            (Some(a), Some(b)) => {
                let aux1 = a;
                let aux3 = self.mem_1;
                let z = if input.v > 50i64 {
                    self.mem_2 + aux3
                } else {
                    self.mem_3
                };
                let aux2 = z;
                (aux2, z, aux3, aux1)
            }
            (Some(a), _) => {
                let x = a > 0i64;
                let z = match () {
                    () if x && !self.mem_4 => a,
                    _ => self.mem_5,
                };
                (self.mem_6, z, self.mem_7, self.mem_8)
            }
            (_, Some(b)) => {
                let x_1 = input.v > 50i64;
                let z = match () {
                    () if x_1 && !self.mem_9 => b,
                    _ => self.mem_10,
                };
                (self.mem_11, z, self.mem_12, self.mem_13)
            }
            (_, _) => (self.mem_14, self.mem_15, self.mem_16, self.mem_17),
        };
        self.mem = z;
        self.mem_1 = aux2;
        self.mem_2 = aux1;
        self.mem_3 = b;
        self.mem_4 = x;
        self.mem_5 = z;
        self.mem_6 = aux2;
        self.mem_7 = aux3;
        self.mem_8 = aux1;
        self.mem_9 = x_1;
        self.mem_10 = z;
        self.mem_11 = aux2;
        self.mem_12 = aux3;
        self.mem_13 = aux1;
        self.mem_14 = aux2;
        self.mem_15 = z;
        self.mem_16 = aux3;
        self.mem_17 = aux1;
        c
    }
}
pub struct DefineEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct DefineEventsState {
    mem: f64,
    mem_1: i64,
}
impl DefineEventsState {
    pub fn init() -> DefineEventsState {
        DefineEventsState {
            mem: Default::default(),
            mem_1: Default::default(),
        }
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
            (_, _) => (self.mem_1, None, None),
        };
        let c = z;
        let d = match (y) {
            (Some(a)) => 0.1,
            _ => self.mem,
        };
        self.mem = d;
        self.mem_1 = z;
        (c, d, x)
    }
}
pub struct FinalTestInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct FinalTestState {
    mem: i64,
    mem_1: bool,
    mem_2: bool,
    mem_3: bool,
    mem_4: i64,
    mem_5: i64,
    mem_6: bool,
}
impl FinalTestState {
    pub fn init() -> FinalTestState {
        FinalTestState {
            mem: Default::default(),
            mem_1: Default::default(),
            mem_2: Default::default(),
            mem_3: false,
            mem_4: Default::default(),
            mem_5: Default::default(),
            mem_6: Default::default(),
        }
    }
    pub fn step(&mut self, input: FinalTestInput) -> (i64, Option<i64>, Option<i64>) {
        let (w, z, y, x, test) = match (input.a, input.b) {
            (Some(a), Some(_)) => {
                let y = Some(());
                let z = if input.v > 50i64 { 1i64 } else { 0i64 };
                (None, z, y, None, self.mem_1)
            }
            (Some(a), _) => {
                let x = Some(2i64);
                let z = 2i64;
                (None, z, None, x, self.mem_2)
            }
            (_, Some(b)) => {
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                let w = match () {
                    () if test && !self.mem_3 => Some(input.v + self.mem_4),
                    _ => None,
                };
                let test = input.v > 50i64;
                let x = Some(2i64);
                (w, z, None, x, test)
            }
            (_, _) => (None, self.mem_5, None, None, self.mem_6),
        };
        let t = match (input.a) {
            (Some(a)) => Some(a + z),
            _ => None,
        };
        let u = match (y, w) {
            (Some(y), Some(w)) => w + 3i64,
            _ => self.mem,
        };
        self.mem = u;
        self.mem_1 = test;
        self.mem_2 = test;
        self.mem_3 = test;
        self.mem_4 = u;
        self.mem_5 = z;
        self.mem_6 = test;
        (u, t, x)
    }
}
