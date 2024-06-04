#![allow(warnings)]

use grust::grust;
pub mod macro_output;
pub mod output;

grust! {
    #![dump = "examples/aeb/src/macro_output.rs", greusot = true]

    // Branking type
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
    function compute_soft_braking_distance(speed: int) -> int {
        return speed * speed / 100;
    }

    // determine braking strategy
    function brakes(distance: int, speed: int) -> Braking {
        let braking_distance: int = compute_soft_braking_distance(speed);
        let response: Braking = if braking_distance < distance
                                then Braking::SoftBrake
                                else Braking::UrgentBrake;
        return response;
    }

    component braking_state(pedest: int!, speed: int) -> (state: Braking)
        requires { 0 <= speed && speed < 50 } // urban limit
        ensures { when p = pedest? => state != Braking::NoBrake } // safety
    {
        state = when d = pedest? then brakes(d, speed)
                timeout Braking::NoBrake otherwise previous_state;
        let previous_state: Braking = Braking::NoBrake fby state;
    }

    // import signal car::speed_km_h                   : int;
    // import event  car::detect::left::pedestrian_l   : int;
    // import event  car::detect::right::pedestrian_r  : int;
    // export signal car::urban::braking::brakes       : Braking;

    // let event pedestrian: timeout(int) = timeout(pedestrian_l, 500);
    // brakes = braking_state(pedestrian, speed_km_h);
}
