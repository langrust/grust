#![allow(warnings)]

use grust::grust;

grust! {
    #![greusot]

    // Safety distance computation
    function safety_distance(sv_v: int, fv_v: int) -> int
        requires { 0 < sv_v && sv_v <= 50 }
        requires { 0 < fv_v && fv_v < sv_v && sv_v - fv_v <= 10 }
        ensures  { 0 <= result && result <= 140 }
    {
        let rho: int = 1; // SV's reaction time
        let b_max: int = 6;
        let sv_d_stop: int = sv_v*rho + sv_v^2/(2*b_max);
        let fv_d_stop: int = fv_v^2/(2*b_max);
        return sv_d_stop - fv_d_stop;
    }

    // Filters the ACC on driver activation and when approaching FV
    component acc(c:bool, d:int, v:int, s:int) -> (b:int)
        requires { c => (0 < s && s <= 50) }
        requires { c => (-s < v && -10 <= v && v < 0) }
        requires { c => d >= s*(1 - v/6) }
        ensures  { 0 <= b && b <= 6 && (c => b > 0) }
        ensures  { c => forall _t: int, 0 < _t && _t <= -v/b
                     => d + v*_t + b*_t^2/2 >= (s-b*_t)*1 +
                            ((s-b*_t)^2 - (s+v)^2)/(2*6) }
    {
        match c {
            true => {
                b = v^2 / (2 * (d - d_safe));
                let d_safe: int = safety_distance(s, fv_v);
                let fv_v: int = s + v;
            },
            false => {
                b = 0;
                let (fv_v: int, d_safe: int) = (0, 0);
            },
        }
    }
}
