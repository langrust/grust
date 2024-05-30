use grust::grust;
pub mod macro_output;

grust! {
    #![dump = "C:/Users/az03049/Documents/gitlab/langrust/grustine/examples/speed_limiter_events/src/macro_output.rs"]

    // # Types

    // Hysterisis for speed.
    struct Hysterisis {
        value: float,
        flag: bool,
    }
    function new_hysterisis(value: float) -> Hysterisis {
        return Hysterisis { value: value, flag: true };
    }

    // Enumerates the kinds of activation resquests.
    enum ActivationResquest { Off, On }

    // Vehicle dynamic control states.
    enum VdcState { On, Off }

    // Vacuum brake states.
    enum VacuumBrakeState { BelowMinLevel, AboveMinLevel }

    // Tells if the driver presses down hard on the accelerator or not.
    enum Kickdown { Activated }

    enum Failure { Entering, Recovered }

    // Speed limiter states.
    enum SpeedLimiter {
        Off,
        On,
        Fail,
    }

    // Speed limiter 'On' states.
    enum SpeedLimiterOn {
        StandBy,
        Actif,
        OverrideVoluntary,
    }

    // # Computation functions

    // Updates the previous hysterisis according to the current speed and the calibration.
    // Determines if the current speed is within regulation.
    function update_hysterisis(prev_hyst: Hysterisis, speed: float, v_set: float) -> Hysterisis {
        let activation_threshold: float = v_set*0.99;
        let deactivation_threshold: float = v_set*0.98;
        let flag: bool = if prev_hyst.flag && speed <= deactivation_threshold
            then false
            else if !prev_hyst.flag && speed >= activation_threshold
                then true
                else prev_hyst.flag;
        let new_hysterisis: Hysterisis = Hysterisis { value: speed, flag: flag };
        return new_hysterisis;
    }

    // Tells if we are in regulation.
    function in_regulation(hysterisis: Hysterisis) -> bool {
        return hysterisis.flag;
    }

    // Gets a diagnostic from the state of the speed limiter.
    function into_diagnostic(to_be_defined: int) -> int {
        return to_be_defined;
    }

    // Threshold for the limit speed requested by the driver.
    function threshold_set_speed(set_speed: float) -> float {
        let ceiled_speed: float = if set_speed > 150.0 then 150.0 else set_speed;
        let grounded_speed: float = if set_speed < 10.0 then 10.0 else ceiled_speed;
        return grounded_speed;
    }

    // # Transition tests functions

    // Speed limiter 'Activation' condition.
    function activation_condition(vacuum_brake_state: VacuumBrakeState, v_set: float) -> bool {
        return vacuum_brake_state != VacuumBrakeState::BelowMinLevel && v_set > 0.0;
    }

    // Speed limiter 'StandBy' condition.
    function standby_condition(vacuum_brake_state: VacuumBrakeState, v_set: float) -> bool {
        return vacuum_brake_state == VacuumBrakeState::BelowMinLevel || v_set <= 0.0;
    }

    // # Components

    // Processes the speed setted by the driver.
    component process_set_speed(set_speed: float?) -> (v_set: float, v_update: bool) {
        let prev_v_set: float = 0.0 fby v_set;
        when {
            v = set_speed => {
                v_set = threshold_set_speed(v);
                v_update = prev_v_set != v_set;
            },
            otherwise => {
                v_set = prev_v_set;
                v_update = false;
            }
        }
    }

    // Speed limiter state machine.
    component speed_limiter(
        activation_req: ActivationResquest?,
        vacuum_brake_state: VacuumBrakeState,
        kickdown: Kickdown?,
        failure: Failure?,
        vdc_disabled: VdcState,
        speed: float,
        v_set: float,
    ) -> (
        state: SpeedLimiter,
        on_state: SpeedLimiterOn,
        in_regulation: bool,
        state_update: bool,
    ) @ 10 ms {
        let prev_state: SpeedLimiter = SpeedLimiter::Off fby state;
        let prev_on_state: SpeedLimiterOn = SpeedLimiterOn::StandBy fby on_state;
        let prev_in_regulation: bool = false fby in_regulation;
        when {
            ActivationResquest::Off = activation_req => {
                state = SpeedLimiter::Off;
                on_state = SpeedLimiterOn::StandBy;
                in_regulation = false;
                state_update = prev_state != SpeedLimiter::Off;
            },
            ActivationResquest::On = activation_req if prev_state == SpeedLimiter::Off => {
                state = SpeedLimiter::On;
                on_state = SpeedLimiterOn::StandBy;
                in_regulation = true;
                state_update = true;
            },
            Failure::Entering = failure => {
                state = SpeedLimiter::Fail;
                on_state = SpeedLimiterOn::StandBy;
                in_regulation = false;
                state_update = prev_state != SpeedLimiter::Fail;
            },
            Failure::Recovered = failure if prev_state == SpeedLimiter::Fail => {
                state = SpeedLimiter::On;
                on_state = SpeedLimiterOn::StandBy;
                in_regulation = true;
                state_update = true;
            },
            otherwise => {
                match prev_state {
                    SpeedLimiter::On => {
                        state = prev_state;
                        (on_state, in_regulation, state_update) = speed_limiter_on(
                            prev_on_state,
                            vacuum_brake_state,
                            kickdown, speed, v_set,
                        );
                    },
                    _ => {
                        state = prev_state;
                        on_state = prev_on_state;
                        in_regulation = prev_in_regulation;
                        state_update = false;
                    },
                }
            }
        }
    }

    // Speed limiter 'On' state machine.
    component speed_limiter_on(
        prev_on_state: SpeedLimiterOn,
        vacuum_brake_state: VacuumBrakeState,
        kickdown: Kickdown?,
        speed: float,
        v_set: float,
    ) -> (
        on_state: SpeedLimiterOn,
        in_reg: bool,
        state_update: bool,
    ) {
        let prev_hysterisis: Hysterisis = new_hysterisis(0.0) fby hysterisis;
        in_reg = in_regulation(hysterisis);
        when {
            _ = kickdown if prev_on_state == SpeedLimiterOn::Actif => {
                on_state = SpeedLimiterOn::OverrideVoluntary;
                let hysterisis: Hysterisis = prev_hysterisis;
                state_update = true;
            },
            otherwise => {
                match prev_on_state {
                    SpeedLimiterOn::StandBy if activation_condition(vacuum_brake_state, v_set) => {
                        on_state = SpeedLimiterOn::Actif;
                        let hysterisis: Hysterisis = new_hysterisis(0.0);
                        state_update = true;
                    },
                    SpeedLimiterOn::OverrideVoluntary if speed <= v_set => {
                        on_state = SpeedLimiterOn::Actif;
                        let hysterisis: Hysterisis = new_hysterisis(0.0);
                        state_update = true;
                    },
                    SpeedLimiterOn::Actif if standby_condition(vacuum_brake_state, v_set) => {
                        on_state = SpeedLimiterOn::StandBy;
                        let hysterisis: Hysterisis = prev_hysterisis;
                        state_update = true;
                    },
                    SpeedLimiterOn::Actif => {
                        on_state = prev_on_state;
                        let hysterisis: Hysterisis = update_hysterisis(prev_hysterisis, speed, v_set);
                        state_update = false;
                    },
                    _ => {
                        on_state = prev_on_state;
                        let hysterisis: Hysterisis = prev_hysterisis;
                        state_update = false;
                    },
                }
            }
        }
    }

    // # Imports
    import event    car::hmi::speed_limiter::activation : ActivationResquest;
    import event    car::hmi::speed_limiter::set_speed  : float;
    import signal   car::adas::speed                    : float;
    import signal   car::adas::vacuum_brake             : VacuumBrakeState;
    import event    car::adas::kickdown                 : Kickdown;
    import event    car::adas::failure                  : Failure;
    import signal   car::adas::vdc                      : VdcState;

    export signal   car::adas::speed_limiter::in_regulation : bool;
    export signal   car::adas::speed_limiter::v_set         : float;

    let (signal v_set_aux: float, signal v_update: bool) = process_set_speed(set_speed);
    let (
        signal state: SpeedLimiter,
        signal on_state: SpeedLimiterOn,
        signal in_regulation_aux: bool,
        signal state_update: bool,
    ) = speed_limiter(
        activation,
        vacuum_brake,
        kickdown,
        failure,
        vdc,
        speed,
        v_set,
    );
    v_set = v_set_aux;
    in_regulation = in_regulation_aux;
}
