#![allow(warnings)]
use grust::grust;

grust! {
    #![mode = demo, dump = "grust/out/aeb.rs"]
    import signal car::speed_km_h                   : float;
    import event  car::detect::left::pedestrian_l   : float;
    import event  car::detect::right::pedestrian_r  : float;
    export signal car::urban::braking::brakes       : Braking;

    // Braking type
    enum Braking {
        NoBrake,
        SoftBrake,
        UrgentBrake,
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

    component braking_state(pedest: float?, timeout_pedestrian: unit?, speed: float)
                        -> (state: Braking)
        requires { 0. <= speed && speed < 55. } // urban limit
        ensures { when _x = pedest? => state != Braking::NoBrake } // safety
    {
        state = when {
            init                        => Braking::NoBrake,
            let d = pedest?             => brakes(d, speed),
            let _ = timeout_pedestrian? => Braking::NoBrake,
        };
    }

    service aeb @ [10, 3000] {
        let event pedestrian: float = merge(pedestrian_l, pedestrian_r);
        let event timeout_pedestrian: unit = timeout(pedestrian, 2000);
        brakes = braking_state(pedestrian, timeout_pedestrian, speed_km_h);
    }
}
