#![allow(warnings)]

use grust::grust;

mod utils {
    fn floor(a: f64) -> i64 {
        a.floor() as i64
    }
}

grust! {
    fn utils::floor(a: float) -> int;

    import signal float_signal: float;
    export signal int_signal: int;

    service test {
        int_signal = floor(float_signal);
    }
}
