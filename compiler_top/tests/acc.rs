compiler_top::prelude! {}

#[test]
fn should_compile_acc() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/acc.rs", demo, propag = "onchange"]

        // Vehicle speed, computed by another service.
        import signal car::state::speed_km_h                : float;
        // Front distance, from radar sensor.
        import signal car::sensors::radar::distance_m       : float;
        // Activation status by the driver.
        import event  car::cluster::acc_active              : Activation;
        // Braking to reach to maintain safety distance.
        export signal car::actuators::control::brakes_m_s   : float;

        // Activation type.
        enum Activation{ On, Off }

        // Derivation component.
        component derive(x: float, t_ms: float) -> (v_s: float) {
            init (t_ms, x) = (0., 0.); // init `last` memories
            v_s = v_ms / 1000.; // convert m/ms into m/s

            let v_ms: float = (x - last x)/dt_ms;
            let dt_ms: float = t_ms - last t_ms;
        }

        // Integration component.
        component integrate(v_s: float, t_ms: float) -> (x: float) {
            init (t_ms, x) = (0., 0.); // init `last` memories
            let v_ms: float = v_s * 1000.; // convert m/s into m/ms

            let unbounded_x: float = last x + v_ms*dt_ms;
            x = if unbounded_x > 10. then 10.
                else (if unbounded_x < -10. then -10. else unbounded_x);

            let dt_ms: float = t_ms - last t_ms;
        }

        // Safety distance computation.
        function safety_distance(sv_v_m_s: float, fv_v_m_s: float) -> float {
            let rho_s: float = 2.;
            let g: float = 9.81;
            let brake_max: float = 0.6*g;
            // distance for SV to stop if it brakes max after a reaction time `rho_s`
            let sv_d_stop_m: float = sv_v_m_s*rho_s + sv_v_m_s*sv_v_m_s/(2.*brake_max);
            // distance for FV to stop if it brakes max
            let fv_d_stop_m: float = fv_v_m_s*fv_v_m_s/(2.*brake_max);
            return sv_d_stop_m - fv_d_stop_m;
        }

        // Command maintaining safety distance.
        //
        // If SV is getting closer to FV, then we need to maintain a safety distance `d_safe`:
        // => find `b_c` such that
        //      ->   sv_v - b_c*t = fv_v
        //      ->   distance(t) = fv_x + fv_v*t - (sv_x + sv_v*t - b_c*t²/2) > d_safe
        // => b_c > (fv_v - sv_v)²/(fv_x - sv_x - d_safe)
        component command(distance_m: float, sv_v_km_h: float, t_ms: float) -> (brakes_command: float) {
            let distancing_m_s: float = derive(distance_m, t_ms);
            brakes_command = distancing_m_s*distancing_m_s / (distance_m - d_safe_m);
            let d_safe_m: float = safety_distance(sv_v_m_s, fv_v_m_s);
            let fv_v_m_s: float = sv_v_m_s + distancing_m_s;
            let sv_v_m_s: float = sv_v_km_h / 3.6;
        }

        // Error on command.
        component error(sv_v_km_h: float, brakes_m_s_command: float, t_ms: float) -> (e_m_s: float) {
            let a_m_s: float = derive(sv_v_m_s, t_ms);
            let sv_v_m_s: float = sv_v_km_h / 3.6;
            e_m_s = a_m_s_command - a_m_s;
            let a_m_s_command: float = -brakes_m_s_command;
        }

        // Proportional Integral Derivative controller.
        component pid(sv_v_km_h: float, b_m_s_command: float, t_ms: float) -> (b_m_s_control: float) {
            let p_e: float = error(sv_v_km_h, b_m_s_command, t_ms);
            let i_e: float = integrate(p_e, t_ms);
            let d_e: float = derive(p_e, t_ms);
            b_m_s_control = 1.*p_e + 0.1*i_e + 0.05*d_e;
        }

        component activate(acc_active: Activation?, distance_m: float) -> (condition: bool) {
            let active: bool = when {
                init => false,
                acc_active? => acc_active == Activation::On,
            };
            init distance_m = 0.;
            let approaching: bool = distance_m < last distance_m;
            condition = active && approaching;
        }

        component filtered_acc(condition: bool, distance_m: float, sv_v_km_h: float, t_ms: float) -> (brakes_m_s: float) {
            match condition {
                true => {
                    let brakes_command_m_s: float = command(distance_m, sv_v_km_h, t_ms);
                    brakes_m_s = pid(sv_v_km_h, brakes_command_m_s, t_ms);
                },
                false => {
                    let brakes_command_m_s: float = 0.;
                    brakes_m_s = 0.;
                },
            }
        }

        // Adaptive Cruise Control
        //
        // This service computes the braking acceleration to perform
        // in order to maintain a safety distance `d_safe` between
        // the subject vehicle (SV) that we control and the front vehicle (FV).
        // In the example bellow, the total `distance` is the sum of `d_safe`
        // and a grace distance `d_grace`.
        //
        //  SV   d_grace       d_safe       FV
        //   x <---------><---------------> x
        //
        // Our goal is to keep `d_grace` above zero, by controlling the brakes.
        service adaptive_cruise_control @ [10, 3000] {
            let signal t: float = time();
            let signal condition: bool = activate(acc_active, distance_m);
            brakes_m_s = filtered_acc(condition, distance_m, speed_km_h, t);
        }
    };
    let (ast, mut ctx) = top.init();
    let tokens = compiler_top::into_token_stream(ast, &mut ctx);
    if let Some(path) = ctx.conf.dump_code {
        compiler_top::dump_code(&path, &tokens).unwrap();
    }
}
