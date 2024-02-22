pub struct TransitiveOInput {
    pub i: i64,
    pub j: i64,
}
pub struct TransitiveOState {}
impl TransitiveOState {
    pub fn init() -> TransitiveOState {
        TransitiveOState {}
    }
    #[requires(input.i<input.j)]
    #[requires(input.j<1000i64)]
    #[ensures(result<1000i64)]
    pub fn step(&mut self, input: TransitiveOInput) -> i64 {
        let o = input.i;
        o
    }
}
