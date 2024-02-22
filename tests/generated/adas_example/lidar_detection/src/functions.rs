pub fn factorial(n: i64) -> i64 {
    let res = if n <= 1i64 { 1i64 } else { n * factorial(n - 1i64) };
    res
}
pub fn fibonacci(n: i64) -> i64 {
    let res = if n <= 0i64 {
        0i64
    } else {
        (if n <= 1i64 { 1i64 } else { fibonacci(n - 1i64) + fibonacci(n - 2i64) })
    };
    res
}
