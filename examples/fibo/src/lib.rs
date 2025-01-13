#![allow(warnings)]

use grust::grust;

grust! {
    component next(i: int) -> (next_o: int) {
        next_o = i + last i init 1;
    }

    component semi_fib(i: int) -> (o: int) {
        let next_o: int = next(i);
        o = last next_o;
    }

    component fib_call() -> (fib: int) {
        let next_o: int = next(fib);
        fib = semi_fib(fib);
    }

    component fib() -> (fib: int) {
        let next_o: int = fib + last fib init 1;
        fib = last next_o;
    }
}
