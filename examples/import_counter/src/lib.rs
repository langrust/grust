#![allow(warnings)]

use grust::grust;

grust! {
    import component counter: (res: bool, tick: bool) -> (o: int);

    function add(x: int, y: int) -> int {
        let res: int = x + y;
        return res;
    }

    component test() -> (y: int) {
        let stop: bool = y > 35;
        init (stop, not_half) = (false, false);
        y = counter(last stop, half);
        let not_half: bool = !half;
        let half: bool = last not_half;
    }
}
