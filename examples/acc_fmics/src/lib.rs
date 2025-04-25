#![allow(warnings)]

use grust::grust;

pub mod utils {
    pub fn convert(x_km_h: f64) -> f64 {
        x_km_h / 3.6
    }
}

grust! {
    #![demo, dump = "examples/acc_fmics/out/mod.rs"]
    use component grust::std::time::derivation::derive(x: float, t: float) -> (i: float);
    use function utils::convert(x_km_h: float) -> float;

    import signal car::state::speed_km_h        : float;
    import signal car::sensors::radar_m         : float;
    import event  car::hmi::acc_active          : Activation;
    export signal car::actuators::brakes_m_s    : float;

    // Activation type
    enum Activation{ On, Off }

    const MIN: int = 10;
    const MAX: int = 3000;

    // Adaptive Cruise Control service
    service adaptive_cruise_control @ [MIN, MAX] {
        let event  radar_e: float = on_change(radar_m);
        let signal cond: bool = activate(acc_active, radar_e);
        let signal speed_m_s: float = convert(speed_km_h);
        let signal vel_delta: float = derive(radar_m, time());
        brakes_m_s = acc(cond, radar_m, vel_delta, speed_m_s);
    }

    const RHO: float = 1.; // reaction time
    const B_MAX: float = 5.886; // 0.6*9.81

    // Safety distance computation
    function safety_distance(sv_v: float, fv_v: float) -> float {
        let sv_d_stop: float = sv_v*RHO + sv_v^2/(2.*B_MAX);
        let fv_d_stop: float = fv_v^2/(2.*B_MAX);
        return sv_d_stop - fv_d_stop;
    }

    // Filters the ACC on driver activation and when approaching FV
    component acc(c:bool, d:float, v:float, s:float) -> (b:float) {
        match c {
            true => {
                b = v^2 / (2.*(d - d_safe));
                let d_safe: float = safety_distance(s, fv_v);
                let fv_v: float = s + v;
            },
            false => {
                b = 0.;
                let (fv_v: float, d_safe: float) = (0., 0.);
            },
        }
    }

    // Activation condition of the ACC
    component activate(act: Activation?, r: float?) -> (c: bool) {
        when {
            init => { d = 0.; active = false; approach = false; }
            act? => { let active: bool = act == Activation::On; }
            r? => { let d: float = r; let approach: bool = d < last d; }
        }
        c = active && approach;
    }
}
