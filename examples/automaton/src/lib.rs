#![allow(warnings)]

use grust::grust;
mod macro_output;

grust! {
    #![dump = "examples/automaton/src/macro_output.rs"]

    enum State {
        On,
        Off,
    }

    function add(x: int, y: int) -> int {
        let res: int = x + y;
        return res;
    }

    component sum(reset: bool, i: int) -> (o: int) {
        o = if reset then 0 else x;
        let x: int = add(0 fby o, i);
    }

    component automaton(switch: bool, i: int) -> (o: int) {
        let state: State = State::Off fby next_state;
        match state {
            State::Off => {
                let next_state: State = if switch then State::On else state;
                let x: int = 0 fby x;
                o = 0;
            },
            State::On => {
                let next_state: State = if switch then State::Off else state;
                let x: int = sum(switch, i);
                o = 10 * x;
            },
        }
    }
}
