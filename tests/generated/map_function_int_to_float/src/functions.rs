pub fn map<F: Fn(i64) -> f64>(x: i64, f: F) -> f64 {
    f(x)
}
