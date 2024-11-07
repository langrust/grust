#[derive(Clone, Copy, PartialEq, Default)]
pub struct Hysterisis {
    pub value: f64,
    pub flag: bool,
}
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ActivationRequest {
    #[default]
    Off,
    On,
    Initialization,
    StandBy,
}
#[derive(Clone, Copy, PartialEq, Default)]
pub enum VdcState {
    #[default]
    On,
    Off,
}
#[derive(Clone, Copy, PartialEq, Default)]
pub enum VacuumBrakeState {
    #[default]
    BelowMinLevel,
    AboveMinLevel,
}
#[derive(Clone, Copy, PartialEq, Default)]
pub enum KickdownState {
    #[default]
    Deactivated,
    Activated,
}
#[derive(Clone, Copy, PartialEq, Default)]
pub enum SpeedLimiter {
    #[default]
    Off,
    On,
    Fail,
}
#[derive(Clone, Copy, PartialEq, Default)]
pub enum SpeedLimiterOn {
    #[default]
    StandBy,
    Actif,
    OverrideVoluntary,
    OverrideInvoluntary,
}
pub fn new_hysterisis(value: f64) -> Hysterisis {
    Hysterisis {
        value: value,
        flag: true,
    }
}
pub fn update_hysterisis(prev_hyst: Hysterisis, speed: f64, v_set: f64) -> Hysterisis {
    let activation_threshold = v_set * 0.99f64;
    let deactivation_threshold = v_set * 0.98f64;
    let flag = if prev_hyst.flag && (speed <= deactivation_threshold) {
        false
    } else {
        if !(prev_hyst.flag) && (speed >= activation_threshold) {
            true
        } else {
            prev_hyst.flag
        }
    };
    let new_hysterisis = Hysterisis {
        value: speed,
        flag: flag,
    };
    new_hysterisis
}
pub fn in_regulation(hysterisis: Hysterisis) -> bool {
    hysterisis.flag
}
pub fn into_diagnostic(to_be_defined: i64) -> i64 {
    to_be_defined
}
pub fn threshold_set_speed(set_speed: f64) -> f64 {
    let ceiled_speed = if set_speed > 150.0f64 {
        150.0f64
    } else {
        set_speed
    };
    let grounded_speed = if set_speed < 10.0f64 {
        10.0f64
    } else {
        ceiled_speed
    };
    grounded_speed
}
pub fn off_condition(activation_req: ActivationRequest, vdc_disabled: VdcState) -> bool {
    (activation_req == ActivationRequest::Off) || (vdc_disabled == VdcState::Off)
}
pub fn on_condition(activation_req: ActivationRequest) -> bool {
    (activation_req == ActivationRequest::On)
        || (activation_req == ActivationRequest::Initialization)
}
pub fn activation_condition(
    activation_req: ActivationRequest,
    vacuum_brake_state: VacuumBrakeState,
    v_set: f64,
) -> bool {
    ((activation_req == ActivationRequest::On)
        && (vacuum_brake_state != VacuumBrakeState::BelowMinLevel))
        && (v_set > 0.0f64)
}
pub fn exit_override_condition(
    activation_req: ActivationRequest,
    kickdown: KickdownState,
    v_set: f64,
    speed: f64,
) -> bool {
    (on_condition(activation_req) && (kickdown != KickdownState::Activated)) && (speed <= v_set)
}
pub fn involuntary_override_condition(
    activation_req: ActivationRequest,
    kickdown: KickdownState,
    v_set: f64,
    speed: f64,
) -> bool {
    (on_condition(activation_req) && (kickdown != KickdownState::Activated)) && (speed > v_set)
}
pub fn voluntary_override_condition(
    activation_req: ActivationRequest,
    kickdown: KickdownState,
) -> bool {
    on_condition(activation_req) && (kickdown == KickdownState::Activated)
}
pub fn standby_condition(
    activation_req: ActivationRequest,
    vacuum_brake_state: VacuumBrakeState,
    v_set: f64,
) -> bool {
    ((activation_req == ActivationRequest::StandBy)
        || (vacuum_brake_state == VacuumBrakeState::BelowMinLevel))
        || (v_set <= 0.0f64)
}
pub struct ProcessSetSpeedInput {
    pub set_speed: f64,
}
pub struct ProcessSetSpeedState {
    last_v_set: f64,
}
impl ProcessSetSpeedState {
    pub fn init() -> ProcessSetSpeedState {
        ProcessSetSpeedState {
            last_v_set: Default::default(),
        }
    }
    pub fn step(&mut self, input: ProcessSetSpeedInput) -> (f64, bool) {
        let v_set = threshold_set_speed(input.set_speed);
        let prev_v_set = self.last_v_set;
        let v_update = prev_v_set != v_set;
        self.last_v_set = v_set;
        (v_set, v_update)
    }
}
pub struct SpeedLimiterOnInput {
    pub prev_on_state: SpeedLimiterOn,
    pub activation_req: ActivationRequest,
    pub vacuum_brake_state: VacuumBrakeState,
    pub kickdown: KickdownState,
    pub speed: f64,
    pub v_set: f64,
}
pub struct SpeedLimiterOnState {
    last_hysterisis: Hysterisis,
}
impl SpeedLimiterOnState {
    pub fn init() -> SpeedLimiterOnState {
        SpeedLimiterOnState {
            last_hysterisis: new_hysterisis(0.0f64),
        }
    }
    pub fn step(&mut self, input: SpeedLimiterOnInput) -> (SpeedLimiterOn, bool) {
        let prev_hysterisis = self.last_hysterisis;
        let (on_state, hysterisis) = match input.prev_on_state {
            SpeedLimiterOn::StandBy
                if activation_condition(
                    input.activation_req,
                    input.vacuum_brake_state,
                    input.v_set,
                ) =>
            {
                let hysterisis = new_hysterisis(0.0f64);
                let on_state = SpeedLimiterOn::Actif;
                (on_state, hysterisis)
            }
            SpeedLimiterOn::OverrideVoluntary
                if exit_override_condition(
                    input.activation_req,
                    input.kickdown,
                    input.v_set,
                    input.speed,
                ) =>
            {
                let hysterisis = new_hysterisis(0.0f64);
                let on_state = SpeedLimiterOn::Actif;
                (on_state, hysterisis)
            }
            SpeedLimiterOn::OverrideInvoluntary
                if exit_override_condition(
                    input.activation_req,
                    input.kickdown,
                    input.v_set,
                    input.speed,
                ) =>
            {
                let hysterisis = new_hysterisis(0.0f64);
                let on_state = SpeedLimiterOn::Actif;
                (on_state, hysterisis)
            }
            SpeedLimiterOn::OverrideVoluntary
                if involuntary_override_condition(
                    input.activation_req,
                    input.kickdown,
                    input.v_set,
                    input.speed,
                ) =>
            {
                let hysterisis = prev_hysterisis;
                let on_state = SpeedLimiterOn::OverrideInvoluntary;
                (on_state, hysterisis)
            }
            SpeedLimiterOn::Actif
                if voluntary_override_condition(input.activation_req, input.kickdown) =>
            {
                let hysterisis = prev_hysterisis;
                let on_state = SpeedLimiterOn::OverrideVoluntary;
                (on_state, hysterisis)
            }
            SpeedLimiterOn::Actif
                if standby_condition(
                    input.activation_req,
                    input.vacuum_brake_state,
                    input.v_set,
                ) =>
            {
                let hysterisis = prev_hysterisis;
                let on_state = SpeedLimiterOn::StandBy;
                (on_state, hysterisis)
            }
            SpeedLimiterOn::Actif => {
                let hysterisis = update_hysterisis(prev_hysterisis, input.speed, input.v_set);
                let on_state = input.prev_on_state;
                (on_state, hysterisis)
            }
            _ => {
                let on_state = input.prev_on_state;
                let hysterisis = prev_hysterisis;
                (on_state, hysterisis)
            }
        };
        let in_reg = in_regulation(hysterisis);
        self.last_hysterisis = hysterisis;
        (on_state, in_reg)
    }
}
pub struct SpeedLimiterInput {
    pub activation_req: ActivationRequest,
    pub vacuum_brake_state: VacuumBrakeState,
    pub kickdown: KickdownState,
    pub vdc_disabled: VdcState,
    pub speed: f64,
    pub v_set: f64,
}
pub struct SpeedLimiterState {
    last_in_regulation: bool,
    last_on_state: SpeedLimiterOn,
    last_state: SpeedLimiter,
    speed_limiter_on: SpeedLimiterOnState,
}
impl SpeedLimiterState {
    pub fn init() -> SpeedLimiterState {
        SpeedLimiterState {
            last_in_regulation: Default::default(),
            last_on_state: Default::default(),
            last_state: Default::default(),
            speed_limiter_on: SpeedLimiterOnState::init(),
        }
    }
    pub fn step(&mut self, input: SpeedLimiterInput) -> (SpeedLimiter, SpeedLimiterOn, bool, bool) {
        let failure = false;
        let prev_state = self.last_state;
        let prev_on_state = self.last_on_state;
        let (in_regulation, on_state, state) = match prev_state {
            _ if off_condition(input.activation_req, input.vdc_disabled) => {
                let state = SpeedLimiter::Off;
                let on_state = prev_on_state;
                let in_regulation = false;
                (in_regulation, on_state, state)
            }
            SpeedLimiter::Off if on_condition(input.activation_req) => {
                let (in_regulation, state, on_state) = match failure {
                    true => {
                        let state = SpeedLimiter::Fail;
                        let on_state = prev_on_state;
                        let in_regulation = false;
                        (in_regulation, state, on_state)
                    }
                    false => {
                        let state = SpeedLimiter::On;
                        let on_state = SpeedLimiterOn::StandBy;
                        let in_regulation = true;
                        (in_regulation, state, on_state)
                    }
                };
                (in_regulation, on_state, state)
            }
            SpeedLimiter::On if failure => {
                let state = SpeedLimiter::Fail;
                let on_state = prev_on_state;
                let in_regulation = false;
                (in_regulation, on_state, state)
            }
            SpeedLimiter::Fail if !(failure) => {
                let state = SpeedLimiter::On;
                let on_state = SpeedLimiterOn::StandBy;
                let in_regulation = true;
                (in_regulation, on_state, state)
            }
            SpeedLimiter::On => {
                let state = prev_state;
                let (on_state, in_regulation) = self.speed_limiter_on.step(SpeedLimiterOnInput {
                    prev_on_state: prev_on_state,
                    activation_req: input.activation_req,
                    vacuum_brake_state: input.vacuum_brake_state,
                    kickdown: input.kickdown,
                    speed: input.speed,
                    v_set: input.v_set,
                });
                (in_regulation, on_state, state)
            }
            _ => {
                let state = prev_state;
                let on_state = prev_on_state;
                let in_regulation = self.last_in_regulation;
                (in_regulation, on_state, state)
            }
        };
        let state_update = (state != prev_state) || (on_state != prev_on_state);
        self.last_in_regulation = in_regulation;
        self.last_on_state = on_state;
        self.last_state = state;
        (state, on_state, in_regulation, state_update)
    }
}
pub mod runtime {
    use super::*;
    use futures::{sink::SinkExt, stream::StreamExt};
    use RuntimeInput as I;
    use RuntimeOutput as O;
    use RuntimeTimer as T;
    #[derive(PartialEq)]
    pub enum RuntimeTimer {
        PeriodSpeedLimiter,
        DelaySpeedLimiter,
        TimeoutSpeedLimiter,
        PeriodSpeedLimiter1,
        DelayAnotherSpeedLimiter,
        TimeoutAnotherSpeedLimiter,
    }
    impl timer_stream::Timing for RuntimeTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::PeriodSpeedLimiter => std::time::Duration::from_millis(10u64),
                T::DelaySpeedLimiter => std::time::Duration::from_millis(10u64),
                T::TimeoutSpeedLimiter => std::time::Duration::from_millis(500u64),
                T::PeriodSpeedLimiter1 => std::time::Duration::from_millis(10u64),
                T::DelayAnotherSpeedLimiter => std::time::Duration::from_millis(10u64),
                T::TimeoutAnotherSpeedLimiter => std::time::Duration::from_millis(500u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::PeriodSpeedLimiter => false,
                T::DelaySpeedLimiter => true,
                T::TimeoutSpeedLimiter => true,
                T::PeriodSpeedLimiter1 => false,
                T::DelayAnotherSpeedLimiter => true,
                T::TimeoutAnotherSpeedLimiter => true,
            }
        }
    }
    pub enum RuntimeInput {
        Activation(ActivationRequest, std::time::Instant),
        VacuumBrake(VacuumBrakeState, std::time::Instant),
        Vdc(VdcState, std::time::Instant),
        SetSpeed(f64, std::time::Instant),
        Speed(f64, std::time::Instant),
        Kickdown(KickdownState, std::time::Instant),
        Timer(T, std::time::Instant),
    }
    impl priority_stream::Reset for RuntimeInput {
        fn do_reset(&self) -> bool {
            match self {
                I::Timer(timer, _) => timer_stream::Timing::do_reset(timer),
                _ => false,
            }
        }
    }
    impl PartialEq for RuntimeInput {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (I::Activation(this, _), I::Activation(other, _)) => this.eq(other),
                (I::VacuumBrake(this, _), I::VacuumBrake(other, _)) => this.eq(other),
                (I::Vdc(this, _), I::Vdc(other, _)) => this.eq(other),
                (I::SetSpeed(this, _), I::SetSpeed(other, _)) => this.eq(other),
                (I::Speed(this, _), I::Speed(other, _)) => this.eq(other),
                (I::Kickdown(this, _), I::Kickdown(other, _)) => this.eq(other),
                (I::Timer(this, _), I::Timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::Activation(_, instant) => *instant,
                I::VacuumBrake(_, instant) => *instant,
                I::Vdc(_, instant) => *instant,
                I::SetSpeed(_, instant) => *instant,
                I::Speed(_, instant) => *instant,
                I::Kickdown(_, instant) => *instant,
                I::Timer(_, instant) => *instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    pub enum RuntimeOutput {
        InRegulation(bool, std::time::Instant),
        VSet(f64, std::time::Instant),
    }
    pub struct Runtime {
        speed_limiter: speed_limiter_service::SpeedLimiterService,
        another_speed_limiter: another_speed_limiter_service::AnotherSpeedLimiterService,
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl Runtime {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let speed_limiter =
                speed_limiter_service::SpeedLimiterService::init(output.clone(), timer.clone());
            let another_speed_limiter =
                another_speed_limiter_service::AnotherSpeedLimiterService::init(
                    output,
                    timer.clone(),
                );
            Runtime {
                speed_limiter,
                another_speed_limiter,
                timer,
            }
        }
        #[inline]
        pub async fn send_timer(
            &mut self,
            timer: T,
            instant: std::time::Instant,
        ) -> Result<(), futures::channel::mpsc::SendError> {
            self.timer.send((timer, instant)).await?;
            Ok(())
        }
        pub async fn run_loop(
            self,
            init_instant: std::time::Instant,
            input: impl futures::Stream<Item = I>,
        ) -> Result<(), futures::channel::mpsc::SendError> {
            futures::pin_mut!(input);
            let mut runtime = self;
            runtime
                .send_timer(T::TimeoutSpeedLimiter, init_instant)
                .await?;
            runtime
                .send_timer(T::TimeoutAnotherSpeedLimiter, init_instant)
                .await?;
            runtime
                .send_timer(T::PeriodSpeedLimiter, init_instant)
                .await?;
            runtime
                .send_timer(T::PeriodSpeedLimiter1, init_instant)
                .await?;
            while let Some(input) = input.next().await {
                match input {
                    I::Timer(T::DelayAnotherSpeedLimiter, instant) => {
                        runtime
                            .another_speed_limiter
                            .handle_delay_another_speed_limiter(instant)
                            .await?;
                    }
                    I::Timer(T::DelaySpeedLimiter, instant) => {
                        runtime
                            .speed_limiter
                            .handle_delay_speed_limiter(instant)
                            .await?;
                    }
                    I::Vdc(vdc, instant) => {
                        runtime.speed_limiter.handle_vdc(instant, vdc).await?;
                        runtime
                            .another_speed_limiter
                            .handle_vdc(instant, vdc)
                            .await?;
                    }
                    I::Timer(T::TimeoutSpeedLimiter, instant) => {
                        runtime
                            .speed_limiter
                            .handle_timeout_speed_limiter(instant)
                            .await?;
                    }
                    I::Kickdown(kickdown, instant) => {
                        runtime
                            .speed_limiter
                            .handle_kickdown(instant, kickdown)
                            .await?;
                        runtime
                            .another_speed_limiter
                            .handle_kickdown(instant, kickdown)
                            .await?;
                    }
                    I::Timer(T::TimeoutAnotherSpeedLimiter, instant) => {
                        runtime
                            .another_speed_limiter
                            .handle_timeout_another_speed_limiter(instant)
                            .await?;
                    }
                    I::SetSpeed(set_speed, instant) => {
                        runtime
                            .speed_limiter
                            .handle_set_speed(instant, set_speed)
                            .await?;
                        runtime
                            .another_speed_limiter
                            .handle_set_speed(instant, set_speed)
                            .await?;
                    }
                    I::Speed(speed, instant) => {
                        runtime.speed_limiter.handle_speed(instant, speed).await?;
                        runtime
                            .another_speed_limiter
                            .handle_speed(instant, speed)
                            .await?;
                    }
                    I::Timer(T::PeriodSpeedLimiter, instant) => {
                        runtime
                            .speed_limiter
                            .handle_period_speed_limiter(instant)
                            .await?;
                    }
                    I::Activation(activation, instant) => {
                        runtime
                            .speed_limiter
                            .handle_activation(instant, activation)
                            .await?;
                        runtime
                            .another_speed_limiter
                            .handle_activation(instant, activation)
                            .await?;
                    }
                    I::Timer(T::PeriodSpeedLimiter1, instant) => {
                        runtime
                            .another_speed_limiter
                            .handle_period_speed_limiter_1(instant)
                            .await?;
                    }
                    I::VacuumBrake(vacuum_brake, instant) => {
                        runtime
                            .speed_limiter
                            .handle_vacuum_brake(instant, vacuum_brake)
                            .await?;
                        runtime
                            .another_speed_limiter
                            .handle_vacuum_brake(instant, vacuum_brake)
                            .await?;
                    }
                }
            }
            Ok(())
        }
    }
    pub mod speed_limiter_service {
        use super::*;
        use futures::{sink::SinkExt, stream::StreamExt};
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct InRegulationAux(bool, bool);
        impl InRegulationAux {
            fn set(&mut self, in_regulation_aux: bool) {
                self.0 = in_regulation_aux;
                self.1 = true;
            }
            fn get(&self) -> bool {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct OnState(SpeedLimiterOn, bool);
        impl OnState {
            fn set(&mut self, on_state: SpeedLimiterOn) {
                self.0 = on_state;
                self.1 = true;
            }
            fn get(&self) -> SpeedLimiterOn {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Vdc(VdcState, bool);
        impl Vdc {
            fn set(&mut self, vdc: VdcState) {
                self.0 = vdc;
                self.1 = true;
            }
            fn get(&self) -> VdcState {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct VacuumBrake(VacuumBrakeState, bool);
        impl VacuumBrake {
            fn set(&mut self, vacuum_brake: VacuumBrakeState) {
                self.0 = vacuum_brake;
                self.1 = true;
            }
            fn get(&self) -> VacuumBrakeState {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct VSet(f64, bool);
        impl VSet {
            fn set(&mut self, v_set: f64) {
                self.0 = v_set;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct VSetAux(f64, bool);
        impl VSetAux {
            fn set(&mut self, v_set_aux: f64) {
                self.0 = v_set_aux;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct SetSpeed(f64, bool);
        impl SetSpeed {
            fn set(&mut self, set_speed: f64) {
                self.0 = set_speed;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Speed(f64, bool);
        impl Speed {
            fn set(&mut self, speed: f64) {
                self.0 = speed;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Activation(ActivationRequest, bool);
        impl Activation {
            fn set(&mut self, activation: ActivationRequest) {
                self.0 = activation;
                self.1 = true;
            }
            fn get(&self) -> ActivationRequest {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct VUpdate(bool, bool);
        impl VUpdate {
            fn set(&mut self, v_update: bool) {
                self.0 = v_update;
                self.1 = true;
            }
            fn get(&self) -> bool {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct State(SpeedLimiter, bool);
        impl State {
            fn set(&mut self, state: SpeedLimiter) {
                self.0 = state;
                self.1 = true;
            }
            fn get(&self) -> SpeedLimiter {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Kickdown(KickdownState, bool);
        impl Kickdown {
            fn set(&mut self, kickdown: KickdownState) {
                self.0 = kickdown;
                self.1 = true;
            }
            fn get(&self) -> KickdownState {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct StateUpdate(bool, bool);
        impl StateUpdate {
            fn set(&mut self, state_update: bool) {
                self.0 = state_update;
                self.1 = true;
            }
            fn get(&self) -> bool {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Context {
            pub in_regulation_aux: InRegulationAux,
            pub on_state: OnState,
            pub vdc: Vdc,
            pub vacuum_brake: VacuumBrake,
            pub v_set: VSet,
            pub v_set_aux: VSetAux,
            pub set_speed: SetSpeed,
            pub speed: Speed,
            pub activation: Activation,
            pub v_update: VUpdate,
            pub state: State,
            pub kickdown: Kickdown,
            pub state_update: StateUpdate,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.in_regulation_aux.reset();
                self.on_state.reset();
                self.vdc.reset();
                self.vacuum_brake.reset();
                self.v_set.reset();
                self.v_set_aux.reset();
                self.set_speed.reset();
                self.speed.reset();
                self.activation.reset();
                self.v_update.reset();
                self.state.reset();
                self.kickdown.reset();
                self.state_update.reset();
            }
        }
        #[derive(Default)]
        pub struct SpeedLimiterServiceStore {
            activation: Option<(ActivationRequest, std::time::Instant)>,
            vacuum_brake: Option<(VacuumBrakeState, std::time::Instant)>,
            period_speed_limiter: Option<((), std::time::Instant)>,
            vdc: Option<(VdcState, std::time::Instant)>,
            set_speed: Option<(f64, std::time::Instant)>,
            speed: Option<(f64, std::time::Instant)>,
            kickdown: Option<(KickdownState, std::time::Instant)>,
        }
        impl SpeedLimiterServiceStore {
            pub fn not_empty(&self) -> bool {
                self.activation.is_some()
                    || self.vacuum_brake.is_some()
                    || self.period_speed_limiter.is_some()
                    || self.vdc.is_some()
                    || self.set_speed.is_some()
                    || self.speed.is_some()
                    || self.kickdown.is_some()
            }
        }
        pub struct SpeedLimiterService {
            context: Context,
            delayed: bool,
            input_store: SpeedLimiterServiceStore,
            speed_limiter: SpeedLimiterState,
            process_set_speed: ProcessSetSpeedState,
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        }
        impl SpeedLimiterService {
            pub fn init(
                output: futures::channel::mpsc::Sender<O>,
                timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
            ) -> SpeedLimiterService {
                let context = Context::init();
                let delayed = true;
                let input_store = Default::default();
                let speed_limiter = SpeedLimiterState::init();
                let process_set_speed = ProcessSetSpeedState::init();
                SpeedLimiterService {
                    context,
                    delayed,
                    input_store,
                    speed_limiter,
                    process_set_speed,
                    output,
                    timer,
                }
            }
            pub async fn handle_activation(
                &mut self,
                activation_instant: std::time::Instant,
                activation: ActivationRequest,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(activation_instant).await?;
                    self.context.reset();
                    self.context.activation.set(activation);
                } else {
                    let unique = self
                        .input_store
                        .activation
                        .replace((activation, activation_instant));
                    assert!(unique.is_none(), "activation changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_vacuum_brake(
                &mut self,
                vacuum_brake_instant: std::time::Instant,
                vacuum_brake: VacuumBrakeState,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(vacuum_brake_instant).await?;
                    self.context.reset();
                    self.context.vacuum_brake.set(vacuum_brake);
                } else {
                    let unique = self
                        .input_store
                        .vacuum_brake
                        .replace((vacuum_brake, vacuum_brake_instant));
                    assert!(unique.is_none(), "vacuum_brake changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_period_speed_limiter(
                &mut self,
                period_speed_limiter_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(period_speed_limiter_instant)
                        .await?;
                    self.context.reset();
                    self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                        .await?;
                    let (v_set_aux, v_update) = self.process_set_speed.step(ProcessSetSpeedInput {
                        set_speed: self.context.set_speed.get(),
                    });
                    self.context.v_set_aux.set(v_set_aux);
                    self.context.v_update.set(v_update);
                    let v_set = self.context.v_set_aux.get();
                    self.context.v_set.set(v_set);
                    self.send_output(O::VSet(v_set, period_speed_limiter_instant))
                        .await?;
                    let (state, on_state, in_regulation_aux, state_update) =
                        self.speed_limiter.step(SpeedLimiterInput {
                            activation_req: self.context.activation.get(),
                            vacuum_brake_state: self.context.vacuum_brake.get(),
                            kickdown: self.context.kickdown.get(),
                            vdc_disabled: self.context.vdc.get(),
                            speed: self.context.speed.get(),
                            v_set: self.context.v_set.get(),
                        });
                    self.context.state.set(state);
                    self.context.on_state.set(on_state);
                    self.context.in_regulation_aux.set(in_regulation_aux);
                    self.context.state_update.set(state_update);
                    let in_regulation = self.context.in_regulation_aux.get();
                    self.send_output(O::InRegulation(in_regulation, period_speed_limiter_instant))
                        .await?;
                } else {
                    let unique = self
                        .input_store
                        .period_speed_limiter
                        .replace(((), period_speed_limiter_instant));
                    assert!(
                        unique.is_none(),
                        "period_speed_limiter changes too frequently"
                    );
                }
                Ok(())
            }
            pub async fn handle_vdc(
                &mut self,
                vdc_instant: std::time::Instant,
                vdc: VdcState,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(vdc_instant).await?;
                    self.context.reset();
                    self.context.vdc.set(vdc);
                } else {
                    let unique = self.input_store.vdc.replace((vdc, vdc_instant));
                    assert!(unique.is_none(), "vdc changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_set_speed(
                &mut self,
                set_speed_instant: std::time::Instant,
                set_speed: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(set_speed_instant).await?;
                    self.context.reset();
                    self.context.set_speed.set(set_speed);
                } else {
                    let unique = self
                        .input_store
                        .set_speed
                        .replace((set_speed, set_speed_instant));
                    assert!(unique.is_none(), "set_speed changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_speed(
                &mut self,
                speed_instant: std::time::Instant,
                speed: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(speed_instant).await?;
                    self.context.reset();
                    self.context.speed.set(speed);
                } else {
                    let unique = self.input_store.speed.replace((speed, speed_instant));
                    assert!(unique.is_none(), "speed changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_timeout_speed_limiter(
                &mut self,
                timeout_speed_limiter_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constrains(timeout_speed_limiter_instant)
                    .await?;
                self.context.reset();
                let (v_set_aux, v_update) = self.process_set_speed.step(ProcessSetSpeedInput {
                    set_speed: self.context.set_speed.get(),
                });
                self.context.v_set_aux.set(v_set_aux);
                self.context.v_update.set(v_update);
                let v_set = self.context.v_set_aux.get();
                self.context.v_set.set(v_set);
                let (state, on_state, in_regulation_aux, state_update) =
                    self.speed_limiter.step(SpeedLimiterInput {
                        activation_req: self.context.activation.get(),
                        vacuum_brake_state: self.context.vacuum_brake.get(),
                        kickdown: self.context.kickdown.get(),
                        vdc_disabled: self.context.vdc.get(),
                        speed: self.context.speed.get(),
                        v_set: self.context.v_set.get(),
                    });
                self.context.state.set(state);
                self.context.on_state.set(on_state);
                self.context.in_regulation_aux.set(in_regulation_aux);
                self.context.state_update.set(state_update);
                let in_regulation = self.context.in_regulation_aux.get();
                self.send_output(O::InRegulation(
                    in_regulation,
                    timeout_speed_limiter_instant,
                ))
                .await?;
                self.send_output(O::VSet(v_set, timeout_speed_limiter_instant))
                    .await?;
                Ok(())
            }
            #[inline]
            pub async fn reset_service_timeout(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer.send((T::TimeoutSpeedLimiter, instant)).await?;
                Ok(())
            }
            pub async fn handle_delay_speed_limiter(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.context.reset();
                if self.input_store.not_empty() {
                    self.reset_time_constrains(instant).await?;
                    match (
                        self.input_store.activation.take(),
                        self.input_store.vacuum_brake.take(),
                        self.input_store.period_speed_limiter.take(),
                        self.input_store.vdc.take(),
                        self.input_store.set_speed.take(),
                        self.input_store.speed.take(),
                        self.input_store.kickdown.take(),
                    ) {
                        (None, None, None, None, None, None, None) => {}
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (None, None, None, Some((vdc, vdc_instant)), None, None, None) => {
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (None, None, None, None, None, Some((speed, speed_instant)), None) => {
                            self.context.speed.set(speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some(((), period_speed_limiter_instant)),
                            Some((vdc, vdc_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.send_timer(T::PeriodSpeedLimiter, period_speed_limiter_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                    }
                } else {
                    self.delayed = true;
                }
                Ok(())
            }
            #[inline]
            pub async fn reset_service_delay(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer.send((T::DelaySpeedLimiter, instant)).await?;
                Ok(())
            }
            pub async fn handle_kickdown(
                &mut self,
                kickdown_instant: std::time::Instant,
                kickdown: KickdownState,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(kickdown_instant).await?;
                    self.context.reset();
                    self.context.kickdown.set(kickdown);
                } else {
                    let unique = self
                        .input_store
                        .kickdown
                        .replace((kickdown, kickdown_instant));
                    assert!(unique.is_none(), "kickdown changes too frequently");
                }
                Ok(())
            }
            #[inline]
            pub async fn reset_time_constrains(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_delay(instant).await?;
                self.reset_service_timeout(instant).await?;
                self.delayed = false;
                Ok(())
            }
            #[inline]
            pub async fn send_output(
                &mut self,
                output: O,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.output.send(output).await?;
                Ok(())
            }
            #[inline]
            pub async fn send_timer(
                &mut self,
                timer: T,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer.send((timer, instant)).await?;
                Ok(())
            }
        }
    }
    pub mod another_speed_limiter_service {
        use super::*;
        use futures::{sink::SinkExt, stream::StreamExt};
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct StateUpdate(bool, bool);
        impl StateUpdate {
            fn set(&mut self, state_update: bool) {
                self.0 = state_update;
                self.1 = true;
            }
            fn get(&self) -> bool {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Kickdown(KickdownState, bool);
        impl Kickdown {
            fn set(&mut self, kickdown: KickdownState) {
                self.0 = kickdown;
                self.1 = true;
            }
            fn get(&self) -> KickdownState {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct VSetAux(f64, bool);
        impl VSetAux {
            fn set(&mut self, v_set_aux: f64) {
                self.0 = v_set_aux;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Activation(ActivationRequest, bool);
        impl Activation {
            fn set(&mut self, activation: ActivationRequest) {
                self.0 = activation;
                self.1 = true;
            }
            fn get(&self) -> ActivationRequest {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct VUpdate(bool, bool);
        impl VUpdate {
            fn set(&mut self, v_update: bool) {
                self.0 = v_update;
                self.1 = true;
            }
            fn get(&self) -> bool {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct InRegulationAux(bool, bool);
        impl InRegulationAux {
            fn set(&mut self, in_regulation_aux: bool) {
                self.0 = in_regulation_aux;
                self.1 = true;
            }
            fn get(&self) -> bool {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct VSet(f64, bool);
        impl VSet {
            fn set(&mut self, v_set: f64) {
                self.0 = v_set;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct SetSpeed(f64, bool);
        impl SetSpeed {
            fn set(&mut self, set_speed: f64) {
                self.0 = set_speed;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct VacuumBrake(VacuumBrakeState, bool);
        impl VacuumBrake {
            fn set(&mut self, vacuum_brake: VacuumBrakeState) {
                self.0 = vacuum_brake;
                self.1 = true;
            }
            fn get(&self) -> VacuumBrakeState {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Speed(f64, bool);
        impl Speed {
            fn set(&mut self, speed: f64) {
                self.0 = speed;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct OnState(SpeedLimiterOn, bool);
        impl OnState {
            fn set(&mut self, on_state: SpeedLimiterOn) {
                self.0 = on_state;
                self.1 = true;
            }
            fn get(&self) -> SpeedLimiterOn {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Vdc(VdcState, bool);
        impl Vdc {
            fn set(&mut self, vdc: VdcState) {
                self.0 = vdc;
                self.1 = true;
            }
            fn get(&self) -> VdcState {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct State(SpeedLimiter, bool);
        impl State {
            fn set(&mut self, state: SpeedLimiter) {
                self.0 = state;
                self.1 = true;
            }
            fn get(&self) -> SpeedLimiter {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Context {
            pub state_update: StateUpdate,
            pub kickdown: Kickdown,
            pub v_set_aux: VSetAux,
            pub activation: Activation,
            pub v_update: VUpdate,
            pub in_regulation_aux: InRegulationAux,
            pub v_set: VSet,
            pub set_speed: SetSpeed,
            pub vacuum_brake: VacuumBrake,
            pub speed: Speed,
            pub on_state: OnState,
            pub vdc: Vdc,
            pub state: State,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.state_update.reset();
                self.kickdown.reset();
                self.v_set_aux.reset();
                self.activation.reset();
                self.v_update.reset();
                self.in_regulation_aux.reset();
                self.v_set.reset();
                self.set_speed.reset();
                self.vacuum_brake.reset();
                self.speed.reset();
                self.on_state.reset();
                self.vdc.reset();
                self.state.reset();
            }
        }
        #[derive(Default)]
        pub struct AnotherSpeedLimiterServiceStore {
            activation: Option<(ActivationRequest, std::time::Instant)>,
            vacuum_brake: Option<(VacuumBrakeState, std::time::Instant)>,
            vdc: Option<(VdcState, std::time::Instant)>,
            period_speed_limiter_1: Option<((), std::time::Instant)>,
            set_speed: Option<(f64, std::time::Instant)>,
            speed: Option<(f64, std::time::Instant)>,
            kickdown: Option<(KickdownState, std::time::Instant)>,
        }
        impl AnotherSpeedLimiterServiceStore {
            pub fn not_empty(&self) -> bool {
                self.activation.is_some()
                    || self.vacuum_brake.is_some()
                    || self.vdc.is_some()
                    || self.period_speed_limiter_1.is_some()
                    || self.set_speed.is_some()
                    || self.speed.is_some()
                    || self.kickdown.is_some()
            }
        }
        pub struct AnotherSpeedLimiterService {
            context: Context,
            delayed: bool,
            input_store: AnotherSpeedLimiterServiceStore,
            process_set_speed: ProcessSetSpeedState,
            speed_limiter: SpeedLimiterState,
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        }
        impl AnotherSpeedLimiterService {
            pub fn init(
                output: futures::channel::mpsc::Sender<O>,
                timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
            ) -> AnotherSpeedLimiterService {
                let context = Context::init();
                let delayed = true;
                let input_store = Default::default();
                let process_set_speed = ProcessSetSpeedState::init();
                let speed_limiter = SpeedLimiterState::init();
                AnotherSpeedLimiterService {
                    context,
                    delayed,
                    input_store,
                    process_set_speed,
                    speed_limiter,
                    output,
                    timer,
                }
            }
            pub async fn handle_activation(
                &mut self,
                activation_instant: std::time::Instant,
                activation: ActivationRequest,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(activation_instant).await?;
                    self.context.reset();
                    self.context.activation.set(activation);
                } else {
                    let unique = self
                        .input_store
                        .activation
                        .replace((activation, activation_instant));
                    assert!(unique.is_none(), "activation changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_vacuum_brake(
                &mut self,
                vacuum_brake_instant: std::time::Instant,
                vacuum_brake: VacuumBrakeState,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(vacuum_brake_instant).await?;
                    self.context.reset();
                    self.context.vacuum_brake.set(vacuum_brake);
                } else {
                    let unique = self
                        .input_store
                        .vacuum_brake
                        .replace((vacuum_brake, vacuum_brake_instant));
                    assert!(unique.is_none(), "vacuum_brake changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_vdc(
                &mut self,
                vdc_instant: std::time::Instant,
                vdc: VdcState,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(vdc_instant).await?;
                    self.context.reset();
                    self.context.vdc.set(vdc);
                } else {
                    let unique = self.input_store.vdc.replace((vdc, vdc_instant));
                    assert!(unique.is_none(), "vdc changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_period_speed_limiter_1(
                &mut self,
                period_speed_limiter_1_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(period_speed_limiter_1_instant)
                        .await?;
                    self.context.reset();
                    self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                        .await?;
                    let (v_set_aux, v_update) = self.process_set_speed.step(ProcessSetSpeedInput {
                        set_speed: self.context.set_speed.get(),
                    });
                    self.context.v_set_aux.set(v_set_aux);
                    self.context.v_update.set(v_update);
                    let v_set = self.context.v_set_aux.get();
                    self.context.v_set.set(v_set);
                    let (state, on_state, in_regulation_aux, state_update) =
                        self.speed_limiter.step(SpeedLimiterInput {
                            activation_req: self.context.activation.get(),
                            vacuum_brake_state: self.context.vacuum_brake.get(),
                            kickdown: self.context.kickdown.get(),
                            vdc_disabled: self.context.vdc.get(),
                            speed: self.context.speed.get(),
                            v_set: self.context.v_set.get(),
                        });
                    self.context.state.set(state);
                    self.context.on_state.set(on_state);
                    self.context.in_regulation_aux.set(in_regulation_aux);
                    self.context.state_update.set(state_update);
                    let in_regulation = self.context.in_regulation_aux.get();
                    self.send_output(O::InRegulation(
                        in_regulation,
                        period_speed_limiter_1_instant,
                    ))
                    .await?;
                    self.send_output(O::VSet(v_set, period_speed_limiter_1_instant))
                        .await?;
                } else {
                    let unique = self
                        .input_store
                        .period_speed_limiter_1
                        .replace(((), period_speed_limiter_1_instant));
                    assert!(
                        unique.is_none(),
                        "period_speed_limiter_1 changes too frequently"
                    );
                }
                Ok(())
            }
            pub async fn handle_timeout_another_speed_limiter(
                &mut self,
                timeout_another_speed_limiter_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constrains(timeout_another_speed_limiter_instant)
                    .await?;
                self.context.reset();
                let (v_set_aux, v_update) = self.process_set_speed.step(ProcessSetSpeedInput {
                    set_speed: self.context.set_speed.get(),
                });
                self.context.v_set_aux.set(v_set_aux);
                self.context.v_update.set(v_update);
                let v_set = self.context.v_set_aux.get();
                self.context.v_set.set(v_set);
                self.send_output(O::VSet(v_set, timeout_another_speed_limiter_instant))
                    .await?;
                let (state, on_state, in_regulation_aux, state_update) =
                    self.speed_limiter.step(SpeedLimiterInput {
                        activation_req: self.context.activation.get(),
                        vacuum_brake_state: self.context.vacuum_brake.get(),
                        kickdown: self.context.kickdown.get(),
                        vdc_disabled: self.context.vdc.get(),
                        speed: self.context.speed.get(),
                        v_set: self.context.v_set.get(),
                    });
                self.context.state.set(state);
                self.context.on_state.set(on_state);
                self.context.in_regulation_aux.set(in_regulation_aux);
                self.context.state_update.set(state_update);
                let in_regulation = self.context.in_regulation_aux.get();
                self.send_output(O::InRegulation(
                    in_regulation,
                    timeout_another_speed_limiter_instant,
                ))
                .await?;
                Ok(())
            }
            #[inline]
            pub async fn reset_service_timeout(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer
                    .send((T::TimeoutAnotherSpeedLimiter, instant))
                    .await?;
                Ok(())
            }
            pub async fn handle_set_speed(
                &mut self,
                set_speed_instant: std::time::Instant,
                set_speed: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(set_speed_instant).await?;
                    self.context.reset();
                    self.context.set_speed.set(set_speed);
                } else {
                    let unique = self
                        .input_store
                        .set_speed
                        .replace((set_speed, set_speed_instant));
                    assert!(unique.is_none(), "set_speed changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_speed(
                &mut self,
                speed_instant: std::time::Instant,
                speed: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(speed_instant).await?;
                    self.context.reset();
                    self.context.speed.set(speed);
                } else {
                    let unique = self.input_store.speed.replace((speed, speed_instant));
                    assert!(unique.is_none(), "speed changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_delay_another_speed_limiter(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.context.reset();
                if self.input_store.not_empty() {
                    self.reset_time_constrains(instant).await?;
                    match (
                        self.input_store.activation.take(),
                        self.input_store.vacuum_brake.take(),
                        self.input_store.vdc.take(),
                        self.input_store.period_speed_limiter_1.take(),
                        self.input_store.set_speed.take(),
                        self.input_store.speed.take(),
                        self.input_store.kickdown.take(),
                    ) {
                        (None, None, None, None, None, None, None) => {}
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (None, None, Some((vdc, vdc_instant)), None, None, None, None) => {
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                            self.send_output(O::VSet(v_set, instant)).await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                            self.send_output(O::VSet(v_set, instant)).await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            None,
                        ) => {
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (None, None, None, None, None, Some((speed, speed_instant)), None) => {
                            self.context.speed.set(speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                            self.send_output(O::VSet(v_set, instant)).await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                            self.send_output(O::VSet(v_set, instant)).await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            None,
                        ) => {
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                            self.send_output(O::VSet(v_set, instant)).await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                            self.send_output(O::VSet(v_set, instant)).await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            None,
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                            self.send_output(O::VSet(v_set, instant)).await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            None,
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            None,
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                        }
                        (
                            None,
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                            self.send_output(O::VSet(v_set, instant)).await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            None,
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            None,
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            None,
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                        (
                            Some((activation, activation_instant)),
                            Some((vacuum_brake, vacuum_brake_instant)),
                            Some((vdc, vdc_instant)),
                            Some(((), period_speed_limiter_1_instant)),
                            Some((set_speed, set_speed_instant)),
                            Some((speed, speed_instant)),
                            Some((kickdown, kickdown_instant)),
                        ) => {
                            self.context.kickdown.set(kickdown);
                            self.context.speed.set(speed);
                            self.context.set_speed.set(set_speed);
                            self.send_timer(T::PeriodSpeedLimiter1, period_speed_limiter_1_instant)
                                .await?;
                            let (v_set_aux, v_update) =
                                self.process_set_speed.step(ProcessSetSpeedInput {
                                    set_speed: self.context.set_speed.get(),
                                });
                            self.context.v_set_aux.set(v_set_aux);
                            self.context.v_update.set(v_update);
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            self.send_output(O::VSet(v_set, instant)).await?;
                            self.context.vdc.set(vdc);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.activation.set(activation);
                            let (state, on_state, in_regulation_aux, state_update) =
                                self.speed_limiter.step(SpeedLimiterInput {
                                    activation_req: self.context.activation.get(),
                                    vacuum_brake_state: self.context.vacuum_brake.get(),
                                    kickdown: self.context.kickdown.get(),
                                    vdc_disabled: self.context.vdc.get(),
                                    speed: self.context.speed.get(),
                                    v_set: self.context.v_set.get(),
                                });
                            self.context.state.set(state);
                            self.context.on_state.set(on_state);
                            self.context.in_regulation_aux.set(in_regulation_aux);
                            self.context.state_update.set(state_update);
                            let in_regulation = self.context.in_regulation_aux.get();
                            self.send_output(O::InRegulation(in_regulation, instant))
                                .await?;
                        }
                    }
                } else {
                    self.delayed = true;
                }
                Ok(())
            }
            #[inline]
            pub async fn reset_service_delay(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer
                    .send((T::DelayAnotherSpeedLimiter, instant))
                    .await?;
                Ok(())
            }
            pub async fn handle_kickdown(
                &mut self,
                kickdown_instant: std::time::Instant,
                kickdown: KickdownState,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(kickdown_instant).await?;
                    self.context.reset();
                    self.context.kickdown.set(kickdown);
                } else {
                    let unique = self
                        .input_store
                        .kickdown
                        .replace((kickdown, kickdown_instant));
                    assert!(unique.is_none(), "kickdown changes too frequently");
                }
                Ok(())
            }
            #[inline]
            pub async fn reset_time_constrains(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_delay(instant).await?;
                self.reset_service_timeout(instant).await?;
                self.delayed = false;
                Ok(())
            }
            #[inline]
            pub async fn send_output(
                &mut self,
                output: O,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.output.send(output).await?;
                Ok(())
            }
            #[inline]
            pub async fn send_timer(
                &mut self,
                timer: T,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer.send((timer, instant)).await?;
                Ok(())
            }
        }
    }
}
