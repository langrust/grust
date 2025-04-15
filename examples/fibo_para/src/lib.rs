#![allow(warnings)]

use grust::grust;

pub mod module {
    pub fn add_i64(n: i64, m: i64) -> i64 {
        n + m
    }
}

grust! {
    #[weight_percent = 5]
    fn module::add_i64(n: int, m: int) -> int;

    #[weight_percent = 10]
    function add(i: int, j: int) -> int {
        return add_i64(i, j);
    }

    #[weight_percent = 12]
    component next(i: int) -> (next_o: int) {
        init i = 1;
        next_o = add(i, last i);
    }

    component semi_fib(i: int) -> (o: int) {
        let next_o: int = next(i);
        o = last next_o;
        init next_o = 0;
    }

    component fib_call() -> (fib: int) {
        let next_o: int = next(fib);
        fib = semi_fib(fib);
    }

    component fib() -> (fib: int) {
        init (fib, next_o) = (1, 0);
        let next_o: int = fib + last fib;
        fib = last next_o;
    }
}
