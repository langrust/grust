use crate::transitive_o::*;
use creusot_contracts::ensures;
use creusot_contracts::requires;
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
    #[requires(input.i<input.j)]
    #[requires(input.j<1000i64)]
    #[ensures(result<1000i64)]
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
