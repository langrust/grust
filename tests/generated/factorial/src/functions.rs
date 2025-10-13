pub fn factorial(n: i64) -> i64 {
    let res = if n <= 1i64 { 1i64 } else { n * factorial(n - 1i64) };
    res
}
