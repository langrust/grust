pub fn min(a: f64, b: f64) -> f64 {
    let test = a > b;
    let result = if test { b } else { a };
    result
}
