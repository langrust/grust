#![allow(warnings)]

use grust::grust;
mod macro_output;

grust! {
    #![dump = "examples/import_counter/src/macro_output.rs"]

    import component counter: (res: bool, tick: bool) -> (o: int);

    function add(x: int, y: int) -> int {
        let res: int = x + y;
        return res;
    }

    component test() -> (y: int) {
        let stop: bool = y > 35;
        y = counter(last stop, half);
        let not_half: bool = !half;
        let half: bool = last not_half;
    }
}
