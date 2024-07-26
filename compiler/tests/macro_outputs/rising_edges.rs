use grust::grust_std::rising_edge::{RisingEdgeInput, RisingEdgeState};
pub struct RisingEdgesInput {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub v: i64,
}
pub struct RisingEdgesState {
    mem: i64,
    rising_edge: RisingEdgeState,
    rising_edge_1: RisingEdgeState,
    rising_edge_2: RisingEdgeState,
    rising_edge_3: RisingEdgeState,
}
impl RisingEdgesState {
    pub fn init() -> RisingEdgesState {
        RisingEdgesState {
            mem: 0i64,
            rising_edge: RisingEdgeState::init(),
            rising_edge_1: RisingEdgeState::init(),
            rising_edge_2: RisingEdgeState::init(),
            rising_edge_3: RisingEdgeState::init(),
        }
    }
    pub fn step(&mut self, input: RisingEdgesInput) -> (i64, f64, Option<i64>) {
        let x_2 = input.v < 40i64;
        let comp_app_rising_edge_1 = self.rising_edge.step(RisingEdgeInput { test: x_2 });
        let x_1 = input.v > 50i64;
        let comp_app_rising_edge = self.rising_edge.step(RisingEdgeInput { test: x_1 });
        let (z, y, x) = match (input.a, input.b) {
            (Some(a), Some(e)) if comp_app_rising_edge => {
                let y = Some(());
                let z = if input.v > 80i64 { e } else { a };
                (z, y, None)
            }
            (Some(a), _) if a != 0i64 && comp_app_rising_edge_1 => {
                let x = Some(2i64);
                let z = 2i64;
                (z, None, x)
            }
            (_, Some(e)) => {
                let x_3 = e < 20i64;
                let comp_app_rising_edge_2 = self.rising_edge.step(RisingEdgeInput { test: x_3 });
                let x = match () {
                    () if comp_app_rising_edge_2 => Some(2i64),
                    _ => None,
                };
                let z = if input.v > 50i64 { 3i64 } else { 4i64 };
                (z, None, x)
            }
            (_, _) => {
                let x_4 = input.v > 50i64;
                let comp_app_rising_edge_3 = self.rising_edge.step(RisingEdgeInput { test: x_4 });
                let z = match () {
                    () if comp_app_rising_edge_3 => input.v + self.mem,
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
        self.mem = c;
        (c, d, x)
    }
}
