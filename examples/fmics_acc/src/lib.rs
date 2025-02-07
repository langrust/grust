use grust::grust;

grust! {
    #![greusot, dump = "examples/fmics_acc/out/grusted.rs", propag = "onchange"]

    import signal car::state::speed_km_h        : float;
    import signal car::sensors::radar_m         : float;
    import event  car::hmi::acc_active          : Activation;
    export signal car::actuators::brakes_m_s    : float;

    // Activation type
    enum Activation{ On, Off }

    // Adaptive Cruise Control service
    service adaptive_cruise_control @ [10, 3000] {
        let signal t_ms: float = time();
        let event  radar_e: float = on_change(radar_m);
        let signal c: bool = activate(acc_active, radar_e);
        let signal v: float = convert(speed_km_h);
        brakes_m_s = acc(c, radar_m, v, t_ms);
    }

    // Safety distance computation
    function safety_distance(sv_v: float, fv_v: float) -> float
        requires { 0. <= sv_v && sv_v <= 50. }
        requires { sv_v - fv_v <= 10. && fv_v <= 50. }
        ensures  { result <= 130. }
    {
        let rho: float = 1.; // SV's reaction time `rho`
        let b_max: float = 0.6*9.81;
        let sv_d_stop: float = sv_v*rho + sv_v*sv_v/(2.*b_max);
        let fv_d_stop: float = fv_v*fv_v/(2.*b_max);
        let d_safe: float = sv_d_stop - fv_d_stop;
        return if d_safe < 0. then 0. else d_safe;
    }

    // Filters the ACC on driver activation and when approaching FV
    component acc(c:bool, d:float, v:float, t:float) -> (b:float)
        requires { c => 0. <= v && v <= 50. }
        requires { c => -10. <= d_dt && d_dt <= 0. }
        requires { c => d >= v*1. - v*d_dt/0.6*9.81 }
        ensures  { c => forall _t: float, 0. < _t && _t <= -d_dt/b
                    => d + d_dt*_t + b*_t*_t/2. >= d_s }
        ensures  { 0. <= b && b <= 0.6*9.81 }
    {
        init (d_dt, fv_v) = (0., 0.);
        match c {
            true => {
                let d_dt: float = derive(d, t/1000.);
                b = d_dt * d_dt / (d - d_s);
                let d_s: float = safety_distance(v, fv_v);
                let fv_v: float = v + d_dt;
            },
            false => {
                let d_dt: float =  last d_dt;
                let d_s: float = 0.;
                b = 0.;
                let fv_v: float = last fv_v;
            },
        }
    }

    // Activation condition of the ACC
    component activate(act: Activation?, r: float?) -> (c: bool) {
        when {
            init => {
                d = 0.;
                active = false;
                approach = false;
            }
            act? => {
                let active: bool = act == Activation::On;
            }
            r? => {
                let d: float = r;
                let approach: bool = d < last d;
            }
        }
        c = active && approach;
    }

    // Derivation component.
    component derive(d: float, t_ms: float) -> (v_s: float)
        requires { t_ms > last t_ms }
    {
        init (t_ms, d) = (0., 0.); // init `last` memories
        v_s = v_ms / 1000.; // convert m/ms into m/s
        let v_ms: float = when {
            init => 0.,
            (d != last d) => (d - last d)/(t_ms - last t_ms),
        };
    }

    // Converts [kilometers per hour] into [meters per seconds].
    component convert(speed_km_h: float) -> (m_s: float) {
        m_s = speed_km_h / 3.6;
    }
}
