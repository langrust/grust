#![allow(warnings)]

use grust::grust;

grust! {
    #![dump = "grust/out/automaton.rs"]

    enum State {
        Off,
        On,
    }

    function add(x: int, y: int) -> int {
        let res: int = x + y;
        return res;
    }

    component sum(reset: bool, i: int) -> (o: int) {
        o = if reset then 0 else x;
        init o = 0;
        let x: int = add(last o, i);
    }

    component automaton(switch: bool, i: int) -> (o: int) {
        init (next_state, x) = (State::Off, 0);
        let state: State = last next_state;
        match state {
            State::Off => {
                let next_state: State = if switch then State::On else state;
                let x: int = last x;
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
