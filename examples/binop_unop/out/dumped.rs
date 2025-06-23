pub fn add_mult(a: i64, b: i64, c: i64) -> i64 {
    (a + b) * c
}
pub fn add_mod(a: f64, b: f64, c: f64) -> f64 {
    (a + b) % c
}
pub fn sub_div(a: i64, b: i64, c: i64) -> i64 {
    (a - b) / c
}
pub fn sub_eq(a: f64, b: f64, c: f64) -> bool {
    (a - b) == c
}
pub fn add_diff(a: i64, b: i64, c: i64) -> bool {
    (a + b) != c
}
pub fn sub_ge(a: f64, b: f64, c: f64) -> bool {
    (a - b) >= c
}
pub fn add_gt(a: i64, b: i64, c: i64) -> bool {
    (a + b) > c
}
pub fn sub_le(a: f64, b: f64, c: f64) -> bool {
    (a - b) <= c
}
pub fn add_lt_neg(a: i64, b: i64, c: i64) -> bool {
    (a + b) < -(c)
}
pub fn le_or_gt(a: f64, b: f64, c: f64) -> bool {
    ((a - b) <= c) || ((a - b) > c)
}
pub fn le_and_ge_not(a: i64, b: i64, c: i64) -> bool {
    ((a - b) <= c) && !((a - b) > c)
}
