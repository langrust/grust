#![allow(warnings)]

use grust::grust;

grust! {
    #![dump = "grust/out/counter.rs"]

    function add(x: int, y: int) -> int {
        let res: int = x + y;
        return res;
    }

    component counter(res: bool, tick: bool) -> (o: int) {
        init o = 0;
        o = if res then 0 else add(last o, inc);
        let inc: int = if tick then 1 else 0;
    }

    component test() -> (y: int) {
        init (stop, not_half) = (false, false);
        let stop: bool = y > 35;
        y = counter(last stop, half);
        let not_half: bool = !half;
        let half: bool = last not_half;
    }
}
