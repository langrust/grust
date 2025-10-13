use crate::transitive_o::*;
pub struct UsingOInput {
    pub i: i64,
    pub j: i64,
}
pub struct UsingOState {
    transitive_o: TransitiveOState,
}
impl UsingOState {
    pub fn init() -> UsingOState {
        UsingOState {
            transitive_o: TransitiveOState::init(),
        }
    }
    pub fn step(&mut self, input: UsingOInput) -> i64 {
        let o = self
            .transitive_o
            .step(TransitiveOInput {
                i: input.i,
                j: input.j,
            });
        o
    }
}
