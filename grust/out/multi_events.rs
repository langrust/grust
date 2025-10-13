pub struct MultipleEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct MultipleEventsOutput {
    pub c: i64,
}
pub struct MultipleEventsState {
    last_aux1: i64,
    last_aux2: i64,
    last_aux3: i64,
    last_x: bool,
    last_z: i64,
}
impl grust::core::Component for MultipleEventsState {
    type Input = MultipleEventsInput;
    type Output = MultipleEventsOutput;
    fn init() -> MultipleEventsState {
        MultipleEventsState {
            last_aux1: 0i64,
            last_aux2: 0i64,
            last_aux3: 0i64,
            last_x: false,
            last_z: 0i64,
        }
    }
    fn step(&mut self, input: MultipleEventsInput) -> MultipleEventsOutput {
        let c = self.last_z;
        let x = input.v > 50i64;
        let y = match () {
            () if x && !(self.last_x) => Some(()),
            () => None,
        };
        let (aux2, z, aux3, aux1) = match (input.a, input.b, y) {
            (Some(a), Some(b), _) => {
                let aux1 = a;
                let aux3 = self.last_aux2;
                let z = if input.v > 50i64 {
                    self.last_aux1 + aux3
                } else {
                    b
                };
                let aux2 = z;
                (aux2, z, aux3, aux1)
            }
            (Some(a), _, _) if a > 0i64 => {
                let (aux1, aux2, aux3) = (self.last_aux1, self.last_aux2, self.last_aux3);
                let z = a;
                (aux2, z, aux3, aux1)
            }
            (_, Some(b), Some(y)) => {
                let (aux1, aux2, aux3) = (self.last_aux1, self.last_aux2, self.last_aux3);
                let z = b;
                (aux2, z, aux3, aux1)
            }
            (_, _, _) => {
                let (aux1, aux2, aux3, z) =
                    (self.last_aux1, self.last_aux2, self.last_aux3, self.last_z);
                (aux2, z, aux3, aux1)
            }
        };
        self.last_aux1 = aux1;
        self.last_aux2 = aux2;
        self.last_aux3 = aux3;
        self.last_x = x;
        self.last_z = z;
        MultipleEventsOutput { c }
    }
}
pub struct DefineEventsInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct DefineEventsOutput {
    pub c: i64,
    pub d: f64,
    pub x: Option<i64>,
}
pub struct DefineEventsState {
    last_d: f64,
    last_z: i64,
}
impl grust::core::Component for DefineEventsState {
    type Input = DefineEventsInput;
    type Output = DefineEventsOutput;
    fn init() -> DefineEventsState {
        DefineEventsState {
            last_d: 0.0f64,
            last_z: 0i64,
        }
    }
    fn step(&mut self, input: DefineEventsInput) -> DefineEventsOutput {
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
                let z = self.last_z;
                (z, None, None)
            }
        };
        let c = z;
        let d = match (y) {
            (Some(a)) => 0.1f64,
            (_) => {
                let d = self.last_d;
                d
            }
        };
        self.last_d = d;
        self.last_z = z;
        DefineEventsOutput { c, d, x }
    }
}
pub struct FinalTestInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct FinalTestOutput {
    pub u: i64,
    pub t: Option<i64>,
    pub x: Option<i64>,
}
pub struct FinalTestState {
    last_test: bool,
    last_u: i64,
    last_z: i64,
}
impl grust::core::Component for FinalTestState {
    type Input = FinalTestInput;
    type Output = FinalTestOutput;
    fn init() -> FinalTestState {
        FinalTestState {
            last_test: false,
            last_u: 0i64,
            last_z: 0i64,
        }
    }
    fn step(&mut self, input: FinalTestInput) -> FinalTestOutput {
        let (z, y, x) = match (input.a, input.b) {
            (Some(a), Some(_)) => {
                let y = Some(());
                let z = if input.v > 50i64 { 1i64 } else { 0i64 };
                (z, y, None)
            }
            (Some(a), _) => {
                let x = Some(2i64);
                let z = 2i64;
                (z, None, x)
            }
            (_, Some(b)) => {
                let x = Some(2i64);
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                (z, None, x)
            }
            (_, _) => {
                let z = self.last_z;
                (z, None, None)
            }
        };
        let t = match (input.a) {
            (Some(a)) => Some(a + z),
            (_) => None,
        };
        let test = input.v > 50i64;
        let w = match () {
            () if test && !(self.last_test) => Some(input.v + self.last_u),
            () => None,
        };
        let u = match (y, w) {
            (Some(y), Some(w)) => w + 3i64,
            (_, _) => {
                let u = self.last_u;
                u
            }
        };
        self.last_test = test;
        self.last_u = u;
        self.last_z = z;
        FinalTestOutput { u, t, x }
    }
}
