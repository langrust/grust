pub struct WitnessPhantomInput {
    pub j: i64,
}
pub struct WitnessPhantomState {}
impl WitnessPhantomState {
    pub fn init() -> WitnessPhantomState {
        WitnessPhantomState {}
    }
    pub fn step(&mut self, input: WitnessPhantomInput) -> i64 {
        let phantom = input.j;
        phantom
    }
}
