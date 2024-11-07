pub struct MultipleEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct MultipleEventsState {
    last_aux1: i64,
    last_aux2: i64,
    last_aux3: i64,
    last_x: bool,
    last_z: i64,
}
impl MultipleEventsState {
    pub fn init() -> MultipleEventsState {
        MultipleEventsState {
            last_aux1: Default::default(),
            last_aux2: Default::default(),
            last_aux3: Default::default(),
            last_x: false,
            last_z: Default::default(),
        }
    }
    pub fn step(&mut self, input: MultipleEventsInput) -> i64 {
        let c = self.last_z;
        let x = input.v > 50i64;
        let y = match () {
            () if x && !(self.last_x) => Some(()),
            _ => None,
        };
        let (aux2, aux3, aux1, z) = match (input.a, input.b, y) {
            (Some(a), Some(b), _) => {
                let aux1 = a;
                let aux3 = self.last_aux2;
                let z = if input.v > 50i64 {
                    self.last_aux1 + aux3
                } else {
                    b
                };
                let aux2 = z;
                (aux2, aux3, aux1, z)
            }
            (Some(a), _, _) if a > 0i64 => {
                let z = a;
                (self.last_aux2, self.last_aux3, self.last_aux1, z)
            }
            (_, Some(b), Some(y)) => {
                let z = b;
                (self.last_aux2, self.last_aux3, self.last_aux1, z)
            }
            (_, _, _) => (self.last_aux2, self.last_aux3, self.last_aux1, self.last_z),
        };
        self.last_aux1 = aux1;
        self.last_aux2 = aux2;
        self.last_aux3 = aux3;
        self.last_x = x;
        self.last_z = z;
        c
    }
}
pub struct DefineEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct DefineEventsState {
    last_d: f64,
    last_z: i64,
}
impl DefineEventsState {
    pub fn init() -> DefineEventsState {
        DefineEventsState {
            last_d: Default::default(),
            last_z: Default::default(),
        }
    }
    pub fn step(&mut self, input: DefineEventsInput) -> (i64, f64, Option<i64>) {
        let (x, y, z) = match (input.a, input.b) {
            (Some(a), Some(e)) => {
                let z = if input.v > 50i64 { e } else { a };
                let y = Some(());
                (None, y, z)
            }
            (Some(_), _) => {
                let z = 2i64;
                let x = Some(2i64);
                (x, None, z)
            }
            (_, Some(_)) => {
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                let x = Some(2i64);
                (x, None, z)
            }
            (_, _) => (None, None, self.last_z),
        };
        let c = z;
        let d = match (y) {
            (Some(a)) => 0.1f64,
            _ => self.last_d,
        };
        self.last_d = d;
        self.last_z = z;
        (c, d, x)
    }
}
pub struct FinalTestInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct FinalTestState {
    last_test: bool,
    last_u: i64,
    last_z: i64,
}
impl FinalTestState {
    pub fn init() -> FinalTestState {
        FinalTestState {
            last_test: false,
            last_u: Default::default(),
            last_z: Default::default(),
        }
    }
    pub fn step(&mut self, input: FinalTestInput) -> (i64, Option<i64>, Option<i64>) {
        let (y, z, x) = match (input.a, input.b) {
            (Some(a), Some(_)) => {
                let z = if input.v > 50i64 { 1i64 } else { 0i64 };
                let y = Some(());
                (y, z, None)
            }
            (Some(a), _) => {
                let x = Some(2i64);
                let z = 2i64;
                (None, z, x)
            }
            (_, Some(b)) => {
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                let x = Some(2i64);
                (None, z, x)
            }
            (_, _) => (None, self.last_z, None),
        };
        let t = match (input.a) {
            (Some(a)) => Some(a + z),
            _ => None,
        };
        let test = input.v > 50i64;
        let w = match () {
            () if test && !(self.last_test) => Some(input.v + self.last_u),
            _ => None,
        };
        let u = match (y, w) {
            (Some(y), Some(w)) => w + 3i64,
            _ => self.last_u,
        };
        self.last_test = test;
        self.last_u = u;
        self.last_z = z;
        (u, t, x)
    }
}
