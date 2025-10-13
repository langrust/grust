use crate::functions::map;
pub struct MainYInput<F: Fn(i64) -> f64> {
    pub x: i64,
    pub f: F,
}
pub struct MainYState {}
impl MainYState {
    pub fn init() -> MainYState {
        MainYState {}
    }
    pub fn step<F: Fn(i64) -> f64>(&mut self, input: MainYInput<F>) -> f64 {
        let y = map(input.x, input.f);
        y
    }
}
