use creusot_contracts::invariant;

pub mod test_o;
pub mod functions;
pub mod typedefs;

fn main() {
    let mut state = test_o::TestOState::init();
    #[invariant(1000i64 >= state.mem_z)]
    #[invariant(state.mem_z > 0i64)]
    loop {
        let (new_state, _o) = state.step(
            test_o::TestOInput { i1: 10, i2: 11 }
        );
        state = new_state;
    }
}
