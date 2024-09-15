#![allow(warnings)]

use grust::grust;
mod macro_output;

grust! {
    #![dump = "examples/fibo/src/macro_output.rs"]

    component next(i: int) -> (next_o: int) {
        next_o = i + last i init 1;
    }

    component semi_fib(i: int) -> (o: int) {
        let next_o: int = next(i);
        o = last next_o;
    }

    component fib_call() -> (fib: int) {
        fib = semi_fib(fib);
    }

    component fib() -> (fib: int) {
        let next_o: int = fib + last fib init 1;
        fib = last next_o;
    }
}
