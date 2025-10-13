pub struct MapYInput<F: Fn(i64) -> f64> {
    pub x: i64,
    pub f: F,
}
pub struct MapYState {}
impl MapYState {
    pub fn init() -> MapYState {
        MapYState {}
    }
    pub fn step<F: Fn(i64) -> f64>(&mut self, input: MapYInput<F>) -> f64 {
        let y = (input.f)(input.x);
        y
    }
}
