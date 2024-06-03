compiler::prelude! {
    ast::Ast,
    conf,
}

#[test]
fn should_compile_aeb() {
    let ast: Ast = syn::parse_quote! {
        #![dump = "tests/macro_outputs/aeb.rs"]

        // Braking type
        enum Braking {
            UrgentBrake,
            SoftBrake,
            NoBrake,
        }

        // Formula: d = 2 * s^2 / (250 * f)
        // d = braking distance in metres (to be calculated).
        // s = speed in km/h.
        // 250 = fixed figure which is always used.
        // f = coefficient of friction, approx. 0.8 on dry asphalt.
        function compute_soft_braking_distance(speed: float) -> float {
            return speed * speed / 100.0;
        }

        // determine braking strategy
        function brakes(distance: float, speed: float) -> Braking {
            let braking_distance: float = compute_soft_braking_distance(speed);
            let response: Braking = if braking_distance < distance
                                    then Braking::SoftBrake
                                    else Braking::UrgentBrake;
            return response;
        }

        component braking_state(pedest: float!, speed: float) -> (state: Braking)
            requires { 0. <= speed && speed < 50. } // urban limit
            //ensures { pedest? => state != NoBrake } // safety
        {
            when {
                d = pedest => {
                    state = brakes(d, speed);
                },
                timeout pedest => {
                    state = Braking::NoBrake;
                },
                otherwise => {
                    state = Braking::NoBrake fby state;
                }
            }
        }

        import signal car::speed_km_h                   : float;
        import event  car::detect::left::pedestrian_l   : float;
        import event  car::detect::right::pedestrian_r  : float;
        export signal car::urban::braking::brakes       : Braking;

        let event pedestrian: timeout(float) = timeout(pedestrian_l, 500);
        brakes = braking_state(pedestrian, speed_km_h);

    };
    let tokens = compiler::into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        compiler::dump_code(&path, &tokens);
    }
}
