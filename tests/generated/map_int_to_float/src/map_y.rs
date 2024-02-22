pub struct MapYInput {
    pub x: i64,
    pub f: impl Fn(i64) -> f64,
}
pub struct MapYState {}
impl MapYState {
    pub fn init() -> MapYState {
        MapYState {}
    }
    pub fn step(&mut self, input: MapYInput) -> f64 {
        let y = input.f(input.x);
        y
    }
}
