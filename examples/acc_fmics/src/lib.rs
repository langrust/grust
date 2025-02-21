#![allow(warnings)]

use grust::grust;

grust! {
    import signal car::state::speed_km_h        : float;
    import signal car::sensors::radar_m         : float;
    import event  car::hmi::acc_active          : Activation;
    export signal car::actuators::brakes_m_s    : float;

    // Activation type
    enum Activation{ On, Off }

    // Adaptive Cruise Control service
    service adaptive_cruise_control @ [10, 3000] {
        let event  radar_e: float = on_change(radar_m);
        let signal cond: bool = activate(acc_active, radar_e);
        let signal speed_m_s: float = convert(speed_km_h);
        let signal vel_delta: float = derive(radar_m, time());
        brakes_m_s = acc(cond, radar_m, vel_delta, speed_m_s);
    }

    // Safety distance computation
    function safety_distance(sv_v: float, fv_v: float) -> float {
        let rho: float = 1.; // SV's reaction time
        let b_max: float = 0.6*9.81;
        let sv_d_stop: float = sv_v*rho + sv_v^2/(2.*b_max);
        let fv_d_stop: float = fv_v^2/(2.*b_max);
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

    // Derivation component.
    component derive(x: float, t_ms: float) -> (v_s: float) {
        init (t_ms, x) = (0., 0.); // init `last` memories
        v_s = v_ms / 1000.; // convert m/ms into m/s

        let v_ms: float = (x - last x)/dt_ms;
        let dt_ms: float = t_ms - last t_ms;
    }

    // Derivation component.
    component convert(x_km_h: float) -> (x_m_s: float) {
        x_m_s = x_km_h / 3.6;
    }
}
