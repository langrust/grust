compiler_top::prelude! {}

#[test]
fn should_compile_aeb_greusot() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/aeb_greusot.rs", mode = greusot]

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
        function compute_soft_braking_distance(speed: int) -> int
            requires { 0 <= speed && speed < 50 }   // urban limit
        {
            return speed * speed / 100;
        }

        // determine braking strategy
        function brakes(distance: int, speed: int) -> Braking
            requires { 0 <= speed && speed < 50 }   // urban limit
            ensures  { result != Braking::NoBrake }  // safety
        {
            let braking_distance: int = compute_soft_braking_distance(speed);
            let response: Braking = if braking_distance < distance
                                    then Braking::SoftBrake
                                    else Braking::UrgentBrake;
            return response;
        }

        component braking_state(pedest: int?, timeout_pedest: unit?, speed: int) -> (state: Braking)
            requires { 0 <= speed && speed < 50 } // urban limit
            ensures  { when p = pedest? => state != Braking::NoBrake } // safety
        {
            when {
                init => {
                    state = Braking::NoBrake;
                }
                let d = pedest? => {
                    state = brakes(d, speed);
                }
                let _ = timeout_pedest? => {
                    state = Braking::NoBrake;
                }
            }
        }
    };
    let (ast, mut ctx) = top.init();
    let tokens = compiler_top::into_token_stream(ast, &mut ctx);
    if let Some(path) = ctx.conf.dump_code {
        compiler_top::dump_code(&path, &tokens).unwrap();
    }
}
