mod sl {
    use grust::grust;

    grust! {
        #![dump = "examples/sl_demo/src/macro_output.rs", test]
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
        enum ActivationResquest { Off, On, Initialization, StandBy }

        // Vehicle dynamic control states.
        enum VdcState { On, Off }

        // Vacuum brake state.
        enum VacuumBrakeState { BelowMinLevel, AboveMinLevel }

        // Tells if the driver presses down hard on the accelerator or not.
        enum KickdownState{ Deactivated, Activated }

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
            OverrideInvoluntary,
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

        // Speed limiter 'Off' condition.
        function off_condition(activation_req: ActivationResquest, vdc_disabled: VdcState) -> bool {
            return activation_req == ActivationResquest::Off || vdc_disabled == VdcState::Off;
        }

        // Speed limiter 'On' condition.
        function on_condition(activation_req: ActivationResquest) -> bool {
            return activation_req == ActivationResquest::On || activation_req == ActivationResquest::Initialization;
        }

        // Speed limiter 'Activation' condition.
        function activation_condition(activation_req: ActivationResquest, vacuum_brake_state: VacuumBrakeState, v_set: float) -> bool {
            return activation_req == ActivationResquest::On
                && vacuum_brake_state != VacuumBrakeState::BelowMinLevel
                && v_set > 0.0;
        }

        // Speed limiter 'Exit Override' condition.
        function exit_override_condition(activation_req: ActivationResquest, kickdown: KickdownState, v_set: float, speed: float) -> bool {
            return on_condition(activation_req) && kickdown != KickdownState::Activated && speed <= v_set;
        }

        // Speed limiter 'Involuntary Override' condition.
        function involuntary_override_condition(activation_req: ActivationResquest, kickdown: KickdownState, v_set: float, speed: float) -> bool {
            return on_condition(activation_req) && kickdown != KickdownState::Activated && speed > v_set;
        }

        // Speed limiter 'Voluntary Override' condition.
        function voluntary_override_condition(activation_req: ActivationResquest, kickdown: KickdownState) -> bool {
            return on_condition(activation_req) && kickdown == KickdownState::Activated;
        }

        // Speed limiter 'StandBy' condition.
        function standby_condition(activation_req: ActivationResquest, vacuum_brake_state: VacuumBrakeState, v_set: float) -> bool {
            return activation_req == ActivationResquest::StandBy
                || vacuum_brake_state == VacuumBrakeState::BelowMinLevel
                || v_set <= 0.0;
        }

        // # Components

        // Processes the speed setted by the driver.
        component process_set_speed(set_speed: float) -> (v_set: float, v_update: bool) {
            v_set = threshold_set_speed(set_speed);
            v_update = prev_v_set != v_set;
            let prev_v_set: float = 0.0 fby v_set;
        }

        // Speed limiter state machine.
        component speed_limiter(
            activation_req: ActivationResquest,
            vacuum_brake_state: VacuumBrakeState,
            kickdown: KickdownState,
            vdc_disabled: VdcState,
            speed: float,
            v_set: float,
        ) -> (
            state: SpeedLimiter,
            on_state: SpeedLimiterOn,
            in_regulation: bool,
            state_update: bool
        ) @ 10 ms {
            let failure: bool = false;
            let prev_state: SpeedLimiter = SpeedLimiter::Off fby state;
            let prev_on_state: SpeedLimiterOn = SpeedLimiterOn::StandBy fby on_state;
            match prev_state {
                _ if off_condition(activation_req, vdc_disabled) => {
                    state = SpeedLimiter::Off;
                    on_state = prev_on_state;
                    in_regulation = false;
                },
                SpeedLimiter::Off if on_condition(activation_req) => {
                    match failure {
                        true => {
                            state = SpeedLimiter::Fail;
                            on_state = prev_on_state;
                            in_regulation = false;
                        },
                        false => {
                            state = SpeedLimiter::On;
                            on_state = SpeedLimiterOn::StandBy;
                            in_regulation = true;
                        },
                    }
                },
                SpeedLimiter::On if failure => {
                    state = SpeedLimiter::Fail;
                    on_state = prev_on_state;
                    in_regulation = false;
                },
                SpeedLimiter::Fail if !failure => {
                    state = SpeedLimiter::On;
                    on_state = SpeedLimiterOn::StandBy;
                    in_regulation = true;
                },
                SpeedLimiter::On => {
                    state = prev_state;
                    (on_state, in_regulation) = speed_limiter_on(
                        prev_on_state, activation_req,
                        vacuum_brake_state,
                        kickdown, speed, v_set,
                    );
                },
                _ => {
                    state = prev_state;
                    on_state = prev_on_state;
                    in_regulation = false fby in_regulation;
                },
            }
            state_update = state != prev_state || on_state != prev_on_state;
        }

        // Speed limiter 'On' state machine.
        component speed_limiter_on(
            prev_on_state: SpeedLimiterOn,
            activation_req: ActivationResquest,
            vacuum_brake_state: VacuumBrakeState,
            kickdown: KickdownState,
            speed: float,
            v_set: float,
        ) -> (
            on_state: SpeedLimiterOn,
            in_reg: bool
        ) {
            let prev_hysterisis: Hysterisis = new_hysterisis(0.0) fby hysterisis;
            in_reg = in_regulation(hysterisis);
            match prev_on_state {
                SpeedLimiterOn::StandBy if activation_condition(activation_req, vacuum_brake_state, v_set) => {
                    on_state = SpeedLimiterOn::Actif;
                    let hysterisis: Hysterisis = new_hysterisis(0.0);
                },
                SpeedLimiterOn::OverrideVoluntary if exit_override_condition(activation_req, kickdown, v_set, speed) => {
                    on_state = SpeedLimiterOn::Actif;
                    let hysterisis: Hysterisis = new_hysterisis(0.0);
                },
                SpeedLimiterOn::OverrideInvoluntary if exit_override_condition(activation_req, kickdown, v_set, speed) => {
                    on_state = SpeedLimiterOn::Actif;
                    let hysterisis: Hysterisis = new_hysterisis(0.0);
                },
                SpeedLimiterOn::OverrideVoluntary if involuntary_override_condition(activation_req, kickdown, v_set, speed) => {
                    on_state = SpeedLimiterOn::OverrideInvoluntary;
                    let hysterisis: Hysterisis = prev_hysterisis;
                },
                SpeedLimiterOn::Actif if voluntary_override_condition(activation_req, kickdown) => {
                    on_state = SpeedLimiterOn::OverrideVoluntary;
                    let hysterisis: Hysterisis = prev_hysterisis;
                },
                SpeedLimiterOn::Actif if standby_condition(activation_req, vacuum_brake_state, v_set) => {
                    on_state = SpeedLimiterOn::StandBy;
                    let hysterisis: Hysterisis = prev_hysterisis;
                },
                SpeedLimiterOn::Actif => {
                    on_state = prev_on_state;
                    let hysterisis: Hysterisis = update_hysterisis(prev_hysterisis, speed, v_set);
                },
                _ => {
                    on_state = prev_on_state;
                    let hysterisis: Hysterisis = prev_hysterisis;
                },
            }
        }

        service speed_limiter {
            // # Imports
            import signal  car::hmi::speed_limiter::activation : ActivationResquest;
            import signal  car::hmi::speed_limiter::set_speed : float;
            import signal  car::adas::speed : float;
            import signal  car::adas::vacuum_brake : VacuumBrakeState;
            import signal  car::adas::kickdown: KickdownState;
            import signal  car::adas::vdc: VdcState;

            export signal car::adas::speed_limiter::in_regulation : bool;
            export signal car::adas::speed_limiter::v_set         : float;

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
                vdc,
                speed,
                v_set,
            );
            v_set = v_set_aux;
            in_regulation = in_regulation_aux;
        }

        service another_speed_limiter {
            // # Imports
            import signal  car::hmi::speed_limiter::activation : ActivationResquest;
            import signal  car::hmi::speed_limiter::set_speed : float;
            import signal  car::adas::speed : float;
            import signal  car::adas::vacuum_brake : VacuumBrakeState;
            import signal  car::adas::kickdown: KickdownState;
            import signal  car::adas::vdc: VdcState;

            export signal car::adas::speed_limiter::in_regulation : bool;
            export signal car::adas::speed_limiter::v_set         : float;

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
                vdc,
                speed,
                v_set,
            );
            v_set = v_set_aux;
            in_regulation = in_regulation_aux;
        }
    }
}

use futures::StreamExt;
use interface::{
    input, output,
    sl_server::{Sl, SlServer},
    Input, Output,
};
use lazy_static::lazy_static;
use priority_stream::prio_stream;
use sl::speed_limiter_service::{
    SpeedLimiterService, SpeedLimiterServiceInput, SpeedLimiterServiceOutput,
    SpeedLimiterServiceTimer,
};
use std::time::{Duration, Instant};
use timer_stream::timer_stream;
use tonic::{transport::Server, Request, Response, Status, Streaming};

// include the `interface` module, which is generated from interface.proto.
pub mod interface {
    tonic::include_proto!("interface");
}

lazy_static! {
    /// Initial instant.
    static ref INIT : Instant = Instant::now();
}

fn into_speed_limiter_service_input(input: Input) -> Option<SpeedLimiterServiceInput> {
    match input.message {
        Some(input::Message::Activation(0)) => Some(SpeedLimiterServiceInput::activation(
            sl::ActivationRequest::On,
            INIT.clone() + Duration::from_millis(input.timestamp as u64),
        )),
        Some(input::Message::Activation(1)) => Some(SpeedLimiterServiceInput::activation(
            sl::ActivationRequest::Off,
            INIT.clone() + Duration::from_millis(input.timestamp as u64),
        )),
        Some(input::Message::SetSpeed(set_speed)) => Some(SpeedLimiterServiceInput::set_speed(
            set_speed,
            INIT.clone() + Duration::from_millis(input.timestamp as u64),
        )),
        Some(input::Message::Speed(speed)) => Some(SpeedLimiterServiceInput::speed(
            speed,
            INIT.clone() + Duration::from_millis(input.timestamp as u64),
        )),
        Some(input::Message::VacuumBrake(0)) => Some(SpeedLimiterServiceInput::vacuum_brake(
            sl::VacuumBrakeState::BelowMinLevel,
            INIT.clone() + Duration::from_millis(input.timestamp as u64),
        )),
        Some(input::Message::VacuumBrake(1)) => Some(SpeedLimiterServiceInput::vacuum_brake(
            sl::VacuumBrakeState::AboveMinLevel,
            INIT.clone() + Duration::from_millis(input.timestamp as u64),
        )),
        Some(input::Message::Kickdown(0)) => Some(SpeedLimiterServiceInput::kickdown(
            sl::Kickdown::Activated,
            INIT.clone() + Duration::from_millis(input.timestamp as u64),
        )),
        Some(input::Message::Failure(0)) => Some(SpeedLimiterServiceInput::failure(
            sl::Failure::Recovered,
            INIT.clone() + Duration::from_millis(input.timestamp as u64),
        )),
        Some(input::Message::Failure(1)) => Some(SpeedLimiterServiceInput::failure(
            sl::Failure::Entering,
            INIT.clone() + Duration::from_millis(input.timestamp as u64),
        )),
        Some(input::Message::Vdc(0)) => Some(SpeedLimiterServiceInput::vdc(
            sl::VdcState::On,
            INIT.clone() + Duration::from_millis(input.timestamp as u64),
        )),
        Some(input::Message::Vdc(1)) => Some(SpeedLimiterServiceInput::vdc(
            sl::VdcState::Off,
            INIT.clone() + Duration::from_millis(input.timestamp as u64),
        )),
        None => None,
        Some(input::Message::Activation(_))
        | Some(input::Message::VacuumBrake(_))
        | Some(input::Message::Kickdown(_))
        | Some(input::Message::Failure(_))
        | Some(input::Message::Vdc(_)) => None,
    }
}

fn from_speed_limiter_service_output(output: SpeedLimiterServiceOutput) -> Result<Output, Status> {
    match output {
        SpeedLimiterServiceOutput::in_regulation(in_regulation, instant) => Ok(Output {
            message: Some(output::Message::InRegulation(in_regulation)),
            timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
        }),
        SpeedLimiterServiceOutput::v_set(v_set, instant) => Ok(Output {
            message: Some(output::Message::VSet(v_set)),
            timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
        }),
        SpeedLimiterServiceOutput::sl_state(sl::SpeedLimiterOn::StandBy, instant) => Ok(Output {
            message: Some(output::Message::SlState(2)),
            timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
        }),
        SpeedLimiterServiceOutput::sl_state(sl::SpeedLimiterOn::Actif, instant) => Ok(Output {
            message: Some(output::Message::SlState(3)),
            timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
        }),
        SpeedLimiterServiceOutput::sl_state(sl::SpeedLimiterOn::OverrideVoluntary, instant) => {
            Ok(Output {
                message: Some(output::Message::SlState(4)),
                timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
            })
        }
    }
}

pub struct SlRuntime;

#[tonic::async_trait]
impl Sl for SlRuntime {
    type RunSLStream = futures::stream::Map<
        futures::channel::mpsc::Receiver<SpeedLimiterServiceOutput>,
        fn(SpeedLimiterServiceOutput) -> Result<Output, Status>,
    >;

    async fn run_sl(
        &self,
        request: Request<Streaming<Input>>,
    ) -> Result<Response<Self::RunSLStream>, Status> {
        let (timers_sink, timers_stream) = futures::channel::mpsc::channel(4);
        let (output_sink, output_stream) = futures::channel::mpsc::channel(4);

        let request_stream = request.into_inner().filter_map(|input| async {
            input.map(into_speed_limiter_service_input).ok().flatten()
        });
        let timers_stream = timer_stream::<_, _, 1>(timers_stream).map(
            |(timer, deadline): (SpeedLimiterServiceTimer, Instant)| {
                SpeedLimiterServiceInput::timer(timer, deadline)
            },
        );
        let input_stream = prio_stream::<_, _, 7>(
            futures::stream::select(request_stream, timers_stream),
            SpeedLimiterServiceInput::order,
        );

        let speed_limiter_service = SpeedLimiterService::new(output_sink, timers_sink);
        tokio::spawn(speed_limiter_service.run_loop(INIT.clone(), input_stream));

        Ok(Response::new(output_stream.map(
            from_speed_limiter_service_output
                as fn(SpeedLimiterServiceOutput) -> Result<Output, Status>,
        )))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    println!("SlServer listening on {}", addr);

    Server::builder()
        .add_service(SlServer::new(SlRuntime))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for ctrl_c")
        })
        .await?;

    Ok(())
}
