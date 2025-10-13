#![allow(warnings)]

use grust::grust;

grust! {
    #![greusot]

    const RHO: int = 1; // reaction time
    const B_MAX: int = 6; // 0.6*9.81

    // Safety distance computation
    function safety_distance(sv_v: int, fv_v: int) -> int
        requires { 0 < sv_v && sv_v <= 50 }
        requires { 0 < fv_v && fv_v < sv_v && sv_v - fv_v <= 10 }
        ensures  { 0 < result && result < 150 }
        ensures  { result == sv_v*RHO + (sv_v^2)/(2*B_MAX) - (fv_v^2)/(2*B_MAX) }
    {
        let sv_d_stop: int = sv_v*RHO + sv_v^2/(2*B_MAX);
        let fv_d_stop: int = fv_v^2/(2*B_MAX);
        return sv_d_stop - fv_d_stop;
    }

    // Filters the ACC on driver activation and when approaching FV
    component acc(c: bool, d: int, v: int, s: int) -> (b: int)
        requires { d < 150 } // radar detection limitation
        requires { c => (0 < s && s <= 50) && (0 < s+v && v < 0 && -v <= 10) } // scope
        // there is enough distance to brake at maximum rate
        requires { c => d - (s*RHO + (s^2)/(2*B_MAX) - (s+v)^2/(2*B_MAX)) > (v^2)/(2*B_MAX) }
        // braking rate is in correct interval
        ensures  { 0 <= b && b <= B_MAX }
        // ensures  { c => forall _t: int, 0 < _t && _t <= -v/b
        //              => d + v*_t + b*_t^2/2 >= (s-b*_t)*RHO +
        //                     ((s-b*_t)^2 - (s+v)^2)/(2*B_MAX) }
    {
        match c {
            true => {
                b = compute_braking(d - d_safe, v);
                let d_safe: int = safety_distance(s, fv_v);
                let fv_v: int = s + v;
            },
            false => {
                b = 0;
                let (fv_v: int, d_safe: int) = (0, 0);
            },
        }
    }

    // Intermediate braking function to pass the proof
    function compute_braking(d_grace: int, v: int) -> int
        requires { (0 < d_grace &&  d_grace < 150) && (v < 0 && -v <= 10) } // scope
        // there is enough distance to brake at maximum rate
        requires { d_grace > (v^2)/(2*B_MAX) }
        // braking rate is in correct interval
        ensures  { 0 <= result && result <= B_MAX }
    {
        return (v^2) / (2 * d_grace);
    }
}
