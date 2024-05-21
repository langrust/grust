#![allow(warnings)]

use grust::grust;
mod macro_output;

grust! {
    #![dump = "C:/Users/az03049/Documents/gitlab/langrust/grustine/examples/counter/src/macro_output.rs"]

    function add(x: int, y: int) -> int {
        let res: int = x + y;
        return res;
    }

    component counter(res: bool, tick: bool) -> (o: int) {
        o = if res then 0 else add(0 fby o, inc);
        let inc: int = if tick then 1 else 0;
    }

    component test() -> (y: int) {
        y = counter(false fby (y > 35), half);
        let half: bool = true fby !half;
    }
}
