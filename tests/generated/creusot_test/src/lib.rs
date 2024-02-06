use creusot_contracts::*;

pub mod test_o;
pub mod functions;
pub mod typedefs;

#[requires(forall<i:usize> i >= 0usize && i@ < inputs@.len() ==> 1000i64 > inputs[i].i1)]
#[requires(forall<i:usize> i >= 0usize && i@ < inputs@.len() ==> inputs[i].i1 > 0i64)]
#[requires(forall<i:usize> i >= 0usize && i@ < inputs@.len() ==> 1000i64 >= inputs[i].i2)]
#[requires(forall<i:usize> i >= 0usize && i@ < inputs@.len() ==> inputs[i].i2 > inputs[i].i2)]
pub fn run(inputs: Vec<test_o::TestOInput>) {
    let mut state = test_o::TestOState::init();
    #[invariant(1000i64 >= state.mem_z)]
    #[invariant(state.mem_z > 0i64)]
    for input in inputs {
        let _o = state.step(input);
    }
}
