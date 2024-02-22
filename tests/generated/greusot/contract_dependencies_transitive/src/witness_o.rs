pub struct WitnessOInput {
    pub i: i64,
}
pub struct WitnessOState {}
impl WitnessOState {
    pub fn init() -> WitnessOState {
        WitnessOState {}
    }
    pub fn step(&mut self, input: WitnessOInput) -> i64 {
        let o = input.i;
        o
    }
}
