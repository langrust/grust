#![allow(warnings)]

use grust::grust;

grust! {
    #![dump = "grust/out/fibo.rs"]

    component next(i: int) -> (next_o: int) {
        init i = 1;
        next_o = i + last i;
    }

    component semi_fib(i: int) -> (o: int) {
        let next_o: int = next(i);
        o = last next_o;
        init next_o = 0;
    }

    component fib_call() -> (fib: int) {
        let next_o: int = next(fib);
        fib = semi_fib(next_o);
    }

    component fib() -> (fib: int) {
        init (fib, next_o) = (1, 0);
        let next_o: int = fib + last fib;
        fib = last next_o;
    }
}
