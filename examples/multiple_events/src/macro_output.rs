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
        MultipleEventsState {
            mem: Default::default(),
        }
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
            (_, _) => self.mem,
        };
        let c = z;
        self.mem = z;
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
use grust::grust_std::rising_edge::{RisingEdgeInput, RisingEdgeState};
pub struct FinalTestInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct FinalTestState {
    mem: i64,
    mem_1: i64,
    mem_2: i64,
    rising_edge: RisingEdgeState,
}
impl FinalTestState {
    pub fn init() -> FinalTestState {
        FinalTestState {
            mem: Default::default(),
            mem_1: 0i64,
            mem_2: Default::default(),
            rising_edge: RisingEdgeState::init(),
        }
    }
    pub fn step(&mut self, input: FinalTestInput) -> (i64, Option<i64>, Option<i64>) {
        let (z, y, x, w) = match (input.a, input.b) {
            (Some(a), Some(_)) => {
                let y = Some(());
                let z = if input.v > 50i64 { 1i64 } else { 0i64 };
                (z, y, None, None)
            }
            (Some(a), _) => {
                let x = Some(2i64);
                let z = 2i64;
                (z, None, x, None)
            }
            (_, Some(b)) => {
                let x_1 = input.v > 50i64;
                let comp_app_rising_edge = self.rising_edge.step(RisingEdgeInput { test: x_1 });
                let w = match () {
                    () if comp_app_rising_edge => Some(input.v + self.mem_1),
                    _ => None,
                };
                let x = Some(2i64);
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                (z, None, x, w)
            }
            (_, _) => (self.mem_2, None, None, None),
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
        self.mem_1 = u;
        self.mem_2 = z;
        (u, t, x)
    }
}
