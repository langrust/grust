pub fn min(a: f64, b: f64) -> f64 {
    let test = a > b;
    let result = if test { b } else { a };
    result
}
use crate::typedefs::GainPID;
pub fn access_k_p(gain: GainPID) -> f64 {
    gain.k_p
}
pub fn access_k_i(gain: GainPID) -> f64 {
    gain.k_i
}
pub fn access_k_d(gain: GainPID) -> f64 {
    gain.k_d
}
