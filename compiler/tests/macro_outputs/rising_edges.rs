use grust::grust_std::rising_edge::{RisingEdgeInput, RisingEdgeState};
pub struct RisingEdgesInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct RisingEdgesState {
    mem: i64,
    mem_1: f64,
    mem_2: i64,
    mem_3: i64,
    mem_4: i64,
    rising_edge: RisingEdgeState,
    rising_edge_1: RisingEdgeState,
    rising_edge_2: RisingEdgeState,
    rising_edge_3: RisingEdgeState,
}
impl RisingEdgesState {
    pub fn init() -> RisingEdgesState {
        RisingEdgesState {
            mem: default::Default(),
            mem_1: default::Default(),
            mem_2: 0i64,
            mem_3: default::Default(),
            mem_4: default::Default(),
            rising_edge: RisingEdgeState::init(),
            rising_edge_1: RisingEdgeState::init(),
            rising_edge_2: RisingEdgeState::init(),
            rising_edge_3: RisingEdgeState::init(),
        }
    }
    pub fn step(&mut self, input: RisingEdgesInput) -> (i64, f64, Option<i64>) {
        let c = match (input.a) {
            (Some(a)) => a,
            _ => self.mem,
        };
        let x_2 = input.v < 40i64;
        let comp_app_rising_edge_1 = self.rising_edge_1.step(RisingEdgeInput { test: x_2 });
        let x_1 = input.v > 50i64;
        let comp_app_rising_edge = self.rising_edge.step(RisingEdgeInput { test: x_1 });
        let (z, y, x) = match (input.a, input.b) {
            (Some(a), Some(e)) if comp_app_rising_edge => {
                let y = Some(());
                let z = if input.v > 80i64 { e } else { a };
                (z, y, None)
            }
            (Some(a), _) if (a != 0i64) && comp_app_rising_edge_1 => {
                let x = Some(2i64);
                let z = 2i64;
                (z, None, x)
            }
            (_, Some(e)) => {
                let x_4 = e < 20i64;
                let comp_app_rising_edge_3 = self.rising_edge_3.step(RisingEdgeInput { test: x_4 });
                let x = match () {
                    () if comp_app_rising_edge_3 => Some(2i64),
                    _ => None,
                };
                let x_3 = input.v > 50i64;
                let comp_app_rising_edge_2 = self.rising_edge_2.step(RisingEdgeInput { test: x_3 });
                let z = match () {
                    () if comp_app_rising_edge_2 => input.v + self.mem_2,
                    _ => self.mem_3,
                };
                (z, None, x)
            }
            (_, _) => (self.mem_4, None, None),
        };
        let d = match (y) {
            (Some(_)) => 0.1,
            _ => self.mem_1,
        };
        self.mem = c;
        self.mem_1 = d;
        self.mem_2 = c;
        self.mem_3 = z;
        self.mem_4 = z;
        (c, d, x)
    }
}
