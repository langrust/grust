#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct Hysterisis {
    pub value: f64,
    pub flag: bool,
}
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ActivationRequest {
    #[default]
    Off,
    On,
}
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum VdcState {
    #[default]
    On,
    Off,
}
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum VacuumBrakeState {
    #[default]
    BelowMinLevel,
    AboveMinLevel,
}
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Kickdown {
    #[default]
    Activated,
    Deactivated,
}
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Failure {
    #[default]
    Entering,
    Recovered,
}
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum SpeedLimiter {
    #[default]
    Off,
    On,
    Fail,
}
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum SpeedLimiterOn {
    #[default]
    StandBy,
    Active,
    OverrideVoluntary,
}
pub fn new_hysterisis(value: f64) -> Hysterisis {
    Hysterisis {
        value: value,
        flag: true,
    }
}
pub fn update_hysterisis(prev_hysterisis: Hysterisis, speed: f64, v_set: f64) -> Hysterisis {
    let activation_threshold = v_set * 0.99f64;
    let deactivation_threshold = v_set * 0.98f64;
    let flag = if prev_hysterisis.flag && (speed <= deactivation_threshold) {
        false
    } else {
        if !(prev_hysterisis.flag) && (speed >= activation_threshold) {
            true
        } else {
            prev_hysterisis.flag
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
pub fn activation_condition(vacuum_brake_state: VacuumBrakeState, v_set: f64) -> bool {
    (vacuum_brake_state != VacuumBrakeState::BelowMinLevel) && (v_set > 0.0f64)
}
pub fn standby_condition(vacuum_brake_state: VacuumBrakeState, v_set: f64) -> bool {
    (vacuum_brake_state == VacuumBrakeState::BelowMinLevel) || (v_set <= 0.0f64)
}
pub struct ProcessSetSpeedInput {
    pub set_speed: Option<f64>,
}
pub struct ProcessSetSpeedOutput {
    pub v_set: f64,
    pub v_update: bool,
}
pub struct ProcessSetSpeedState {
    last_v_set: f64,
}
impl grust::core::Component for ProcessSetSpeedState {
    type Input = ProcessSetSpeedInput;
    type Output = ProcessSetSpeedOutput;
    fn init() -> ProcessSetSpeedState {
        ProcessSetSpeedState { last_v_set: 0.0f64 }
    }
    fn step(&mut self, input: ProcessSetSpeedInput) -> ProcessSetSpeedOutput {
        let prev_v_set = self.last_v_set;
        let v_set = match (input.set_speed) {
            (Some(v)) => threshold_set_speed(v),
            (_) => {
                let v_set = self.last_v_set;
                v_set
            }
        };
        let v_update = prev_v_set != v_set;
        self.last_v_set = v_set;
        ProcessSetSpeedOutput { v_set, v_update }
    }
}
pub struct SpeedLimiterOnInput {
    pub prev_on_state: SpeedLimiterOn,
    pub vacuum_brake_state: VacuumBrakeState,
    pub kickdown: Option<Kickdown>,
    pub speed: f64,
    pub v_set: f64,
}
pub struct SpeedLimiterOnOutput {
    pub on_state: SpeedLimiterOn,
    pub in_reg: bool,
    pub state_update: bool,
}
pub struct SpeedLimiterOnState {
    last_hysterisis: Hysterisis,
    last_kickdown_state: Kickdown,
}
impl grust::core::Component for SpeedLimiterOnState {
    type Input = SpeedLimiterOnInput;
    type Output = SpeedLimiterOnOutput;
    fn init() -> SpeedLimiterOnState {
        SpeedLimiterOnState {
            last_hysterisis: new_hysterisis(0.0f64),
            last_kickdown_state: Kickdown::Deactivated,
        }
    }
    fn step(&mut self, input: SpeedLimiterOnInput) -> SpeedLimiterOnOutput {
        let prev_hysterisis = self.last_hysterisis;
        let kickdown_state = match (input.kickdown) {
            (Some(Kickdown::Activated)) if input.prev_on_state == SpeedLimiterOn::Active => {
                Kickdown::Activated
            }
            (Some(Kickdown::Deactivated)) => Kickdown::Deactivated,
            (_) => {
                let kickdown_state = self.last_kickdown_state;
                kickdown_state
            }
        };
        let (hysterisis, on_state) = match input.prev_on_state {
            _ if kickdown_state == Kickdown::Activated => {
                let on_state = SpeedLimiterOn::OverrideVoluntary;
                let hysterisis = prev_hysterisis;
                (hysterisis, on_state)
            }
            SpeedLimiterOn::StandBy
                if activation_condition(input.vacuum_brake_state, input.v_set) =>
            {
                let on_state = SpeedLimiterOn::Active;
                let hysterisis = new_hysterisis(0.0f64);
                (hysterisis, on_state)
            }
            SpeedLimiterOn::OverrideVoluntary if input.speed <= input.v_set => {
                let on_state = SpeedLimiterOn::Active;
                let hysterisis = new_hysterisis(0.0f64);
                (hysterisis, on_state)
            }
            SpeedLimiterOn::Active if standby_condition(input.vacuum_brake_state, input.v_set) => {
                let on_state = SpeedLimiterOn::StandBy;
                let hysterisis = prev_hysterisis;
                (hysterisis, on_state)
            }
            SpeedLimiterOn::Active => {
                let on_state = input.prev_on_state;
                let hysterisis = update_hysterisis(prev_hysterisis, input.speed, input.v_set);
                (hysterisis, on_state)
            }
            _ => {
                let on_state = input.prev_on_state;
                let hysterisis = prev_hysterisis;
                (hysterisis, on_state)
            }
        };
        let state_update = input.prev_on_state != on_state;
        let in_reg = in_regulation(hysterisis);
        self.last_hysterisis = hysterisis;
        self.last_kickdown_state = kickdown_state;
        SpeedLimiterOnOutput {
            on_state,
            in_reg,
            state_update,
        }
    }
}
pub struct SpeedLimiterInput {
    pub activation_req: Option<ActivationRequest>,
    pub vacuum_brake_state: VacuumBrakeState,
    pub kickdown: Option<Kickdown>,
    pub failure: Option<Failure>,
    pub speed: f64,
    pub v_set: f64,
}
pub struct SpeedLimiterOutput {
    pub state: SpeedLimiter,
    pub on_state: SpeedLimiterOn,
    pub in_regulation: bool,
    pub state_update: bool,
}
pub struct SpeedLimiterState {
    last_on_state: SpeedLimiterOn,
    last_state: SpeedLimiter,
    speed_limiter_on: SpeedLimiterOnState,
}
impl grust::core::Component for SpeedLimiterState {
    type Input = SpeedLimiterInput;
    type Output = SpeedLimiterOutput;
    fn init() -> SpeedLimiterState {
        SpeedLimiterState {
            last_on_state: SpeedLimiterOn::StandBy,
            last_state: SpeedLimiter::Off,
            speed_limiter_on: <SpeedLimiterOnState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: SpeedLimiterInput) -> SpeedLimiterOutput {
        let prev_state = self.last_state;
        let prev_on_state = self.last_on_state;
        let state = match (input.activation_req, input.failure) {
            (Some(activation_req), _) if activation_req == ActivationRequest::Off => {
                SpeedLimiter::Off
            }
            (Some(ActivationRequest::On), _) if prev_state == SpeedLimiter::Off => SpeedLimiter::On,
            (_, Some(failure)) if failure == Failure::Entering => SpeedLimiter::Fail,
            (_, Some(f)) if (f == Failure::Recovered) && (prev_state == SpeedLimiter::Fail) => {
                SpeedLimiter::On
            }
            (_, _) => {
                let state = self.last_state;
                state
            }
        };
        let (state_update, on_state, in_regulation) = match prev_state {
            SpeedLimiter::On => {
                let (on_state, in_regulation, state_update) = {
                    let SpeedLimiterOnOutput {
                        on_state,
                        in_reg,
                        state_update,
                    } = <SpeedLimiterOnState as grust::core::Component>::step(
                        &mut self.speed_limiter_on,
                        SpeedLimiterOnInput {
                            prev_on_state: prev_on_state,
                            vacuum_brake_state: input.vacuum_brake_state,
                            kickdown: input.kickdown,
                            speed: input.speed,
                            v_set: input.v_set,
                        },
                    );
                    (on_state, in_reg, state_update)
                };
                (state_update, on_state, in_regulation)
            }
            _ => {
                let on_state = SpeedLimiterOn::StandBy;
                let in_regulation = false;
                let state_update = prev_state != state;
                (state_update, on_state, in_regulation)
            }
        };
        self.last_on_state = on_state;
        self.last_state = state;
        SpeedLimiterOutput {
            state,
            on_state,
            in_regulation,
            state_update,
        }
    }
}
pub mod runtime {
    use super::*;
    use grust::futures::{sink::SinkExt, stream::StreamExt};
    #[derive(Debug)]
    pub enum RuntimeInput {
        Failure(Failure, std::time::Instant),
        Speed(f64, std::time::Instant),
        VacuumBrake(VacuumBrakeState, std::time::Instant),
        Activation(ActivationRequest, std::time::Instant),
        Kickdown(Kickdown, std::time::Instant),
        SetSpeed(f64, std::time::Instant),
    }
    use RuntimeInput as I;
    impl grust::core::priority_stream::Reset for RuntimeInput {
        fn do_reset(&self) -> bool {
            match self {
                _ => false,
            }
        }
    }
    impl PartialEq for RuntimeInput {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (I::Failure(this, _), I::Failure(other, _)) => this.eq(other),
                (I::Speed(this, _), I::Speed(other, _)) => this.eq(other),
                (I::VacuumBrake(this, _), I::VacuumBrake(other, _)) => this.eq(other),
                (I::Activation(this, _), I::Activation(other, _)) => this.eq(other),
                (I::Kickdown(this, _), I::Kickdown(other, _)) => this.eq(other),
                (I::SetSpeed(this, _), I::SetSpeed(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::Failure(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Speed(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::VacuumBrake(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Activation(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Kickdown(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::SetSpeed(_, _grust_reserved_instant) => *_grust_reserved_instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    #[derive(Debug, PartialEq)]
    pub enum RuntimeOutput {
        InRegulation(bool, std::time::Instant),
        VSet(f64, std::time::Instant),
    }
    use RuntimeOutput as O;
    #[derive(Debug, Default)]
    pub struct RuntimeInit {
        pub speed: f64,
        pub vacuum_brake: VacuumBrakeState,
        pub set_speed: f64,
    }
    pub struct Runtime {
        _grust_reserved_init_instant: std::time::Instant,
        speed_limiter: speed_limiter_service::SpeedLimiterService,
        output: grust::futures::channel::mpsc::Sender<O>,
    }
    impl Runtime {
        pub fn new(
            _grust_reserved_init_instant: std::time::Instant,
            output: grust::futures::channel::mpsc::Sender<O>,
        ) -> Runtime {
            let speed_limiter = speed_limiter_service::SpeedLimiterService::init(
                _grust_reserved_init_instant,
                output.clone(),
            );
            Runtime {
                _grust_reserved_init_instant,
                speed_limiter,
                output,
            }
        }
        pub async fn run_loop(
            self,
            input: impl grust::futures::Stream<Item = I>,
            init_vals: RuntimeInit,
        ) -> Result<(), grust::futures::channel::mpsc::SendError> {
            grust::futures::pin_mut!(input);
            let mut runtime = self;
            let RuntimeInit {
                speed,
                vacuum_brake,
                set_speed,
            } = init_vals;
            runtime
                .speed_limiter
                .handle_init(speed, vacuum_brake, set_speed)
                .await?;
            while let Some(input) = input.next().await {
                match input {
                    I::Failure(failure, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_failure(_grust_reserved_instant, failure)
                            .await?;
                    }
                    I::Speed(speed, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_speed(_grust_reserved_instant, speed)
                            .await?;
                    }
                    I::VacuumBrake(vacuum_brake, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_vacuum_brake(_grust_reserved_instant, vacuum_brake)
                            .await?;
                    }
                    I::Activation(activation, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_activation(_grust_reserved_instant, activation)
                            .await?;
                    }
                    I::Kickdown(kickdown, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_kickdown(_grust_reserved_instant, kickdown)
                            .await?;
                    }
                    I::SetSpeed(set_speed, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_set_speed(_grust_reserved_instant, set_speed)
                            .await?;
                    }
                }
            }
            Ok(())
        }
    }
    pub mod speed_limiter_service {
        use super::*;
        use grust::futures::{sink::SinkExt, stream::StreamExt};
        mod ctx_ty {
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct StateUpdate(bool, bool);
            impl StateUpdate {
                pub fn set(&mut self, state_update: bool) {
                    self.1 = self.0 != state_update;
                    self.0 = state_update;
                }
                pub fn get(&self) -> bool {
                    self.0
                }
                pub fn take(&mut self) -> bool {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct VUpdate(bool, bool);
            impl VUpdate {
                pub fn set(&mut self, v_update: bool) {
                    self.1 = self.0 != v_update;
                    self.0 = v_update;
                }
                pub fn get(&self) -> bool {
                    self.0
                }
                pub fn take(&mut self) -> bool {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct VacuumBrake(super::VacuumBrakeState, bool);
            impl VacuumBrake {
                pub fn set(&mut self, vacuum_brake: super::VacuumBrakeState) {
                    self.1 = self.0 != vacuum_brake;
                    self.0 = vacuum_brake;
                }
                pub fn get(&self) -> super::VacuumBrakeState {
                    self.0
                }
                pub fn take(&mut self) -> super::VacuumBrakeState {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct Speed(f64, bool);
            impl Speed {
                pub fn set(&mut self, speed: f64) {
                    self.1 = self.0 != speed;
                    self.0 = speed;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct ChangedSetSpeedOld(f64, bool);
            impl ChangedSetSpeedOld {
                pub fn set(&mut self, changed_set_speed_old: f64) {
                    self.1 = self.0 != changed_set_speed_old;
                    self.0 = changed_set_speed_old;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct SetSpeed(f64, bool);
            impl SetSpeed {
                pub fn set(&mut self, set_speed: f64) {
                    self.1 = self.0 != set_speed;
                    self.0 = set_speed;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct VSetAux(f64, bool);
            impl VSetAux {
                pub fn set(&mut self, v_set_aux: f64) {
                    self.1 = self.0 != v_set_aux;
                    self.0 = v_set_aux;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct InRegulationOld(bool, bool);
            impl InRegulationOld {
                pub fn set(&mut self, in_regulation_old: bool) {
                    self.1 = self.0 != in_regulation_old;
                    self.0 = in_regulation_old;
                }
                pub fn get(&self) -> bool {
                    self.0
                }
                pub fn take(&mut self) -> bool {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct VSet(f64, bool);
            impl VSet {
                pub fn set(&mut self, v_set: f64) {
                    self.1 = self.0 != v_set;
                    self.0 = v_set;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct InRegulationAux(bool, bool);
            impl InRegulationAux {
                pub fn set(&mut self, in_regulation_aux: bool) {
                    self.1 = self.0 != in_regulation_aux;
                    self.0 = in_regulation_aux;
                }
                pub fn get(&self) -> bool {
                    self.0
                }
                pub fn take(&mut self) -> bool {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct X(f64, bool);
            impl X {
                pub fn set(&mut self, x: f64) {
                    self.1 = self.0 != x;
                    self.0 = x;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct OnState(super::SpeedLimiterOn, bool);
            impl OnState {
                pub fn set(&mut self, on_state: super::SpeedLimiterOn) {
                    self.1 = self.0 != on_state;
                    self.0 = on_state;
                }
                pub fn get(&self) -> super::SpeedLimiterOn {
                    self.0
                }
                pub fn take(&mut self) -> super::SpeedLimiterOn {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct State(super::SpeedLimiter, bool);
            impl State {
                pub fn set(&mut self, state: super::SpeedLimiter) {
                    self.1 = self.0 != state;
                    self.0 = state;
                }
                pub fn get(&self) -> super::SpeedLimiter {
                    self.0
                }
                pub fn take(&mut self) -> super::SpeedLimiter {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
        }
        #[derive(Clone, Copy, PartialEq, Default, Debug)]
        pub struct Context {
            pub state_update: ctx_ty::StateUpdate,
            pub v_update: ctx_ty::VUpdate,
            pub vacuum_brake: ctx_ty::VacuumBrake,
            pub speed: ctx_ty::Speed,
            pub changed_set_speed_old: ctx_ty::ChangedSetSpeedOld,
            pub set_speed: ctx_ty::SetSpeed,
            pub v_set_aux: ctx_ty::VSetAux,
            pub in_regulation_old: ctx_ty::InRegulationOld,
            pub v_set: ctx_ty::VSet,
            pub in_regulation_aux: ctx_ty::InRegulationAux,
            pub x: ctx_ty::X,
            pub on_state: ctx_ty::OnState,
            pub state: ctx_ty::State,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.state_update.reset();
                self.v_update.reset();
                self.vacuum_brake.reset();
                self.speed.reset();
                self.changed_set_speed_old.reset();
                self.set_speed.reset();
                self.v_set_aux.reset();
                self.in_regulation_old.reset();
                self.v_set.reset();
                self.in_regulation_aux.reset();
                self.x.reset();
                self.on_state.reset();
                self.state.reset();
            }
        }
        #[derive(Default)]
        pub struct SpeedLimiterServiceStore {
            failure: Option<(Failure, std::time::Instant)>,
            speed: Option<(f64, std::time::Instant)>,
            vacuum_brake: Option<(VacuumBrakeState, std::time::Instant)>,
            activation: Option<(ActivationRequest, std::time::Instant)>,
            kickdown: Option<(Kickdown, std::time::Instant)>,
            set_speed: Option<(f64, std::time::Instant)>,
        }
        impl SpeedLimiterServiceStore {
            pub fn not_empty(&self) -> bool {
                self.failure.is_some()
                    || self.speed.is_some()
                    || self.vacuum_brake.is_some()
                    || self.activation.is_some()
                    || self.kickdown.is_some()
                    || self.set_speed.is_some()
            }
        }
        pub struct SpeedLimiterService {
            _grust_reserved_init_instant: std::time::Instant,
            context: Context,
            delayed: bool,
            input_store: SpeedLimiterServiceStore,
            process_set_speed: ProcessSetSpeedState,
            speed_limiter: SpeedLimiterState,
            output: grust::futures::channel::mpsc::Sender<O>,
        }
        impl SpeedLimiterService {
            pub fn init(
                _grust_reserved_init_instant: std::time::Instant,
                output: grust::futures::channel::mpsc::Sender<O>,
            ) -> SpeedLimiterService {
                let context = Context::init();
                let delayed = true;
                let input_store = Default::default();
                let process_set_speed = <ProcessSetSpeedState as grust::core::Component>::init();
                let speed_limiter = <SpeedLimiterState as grust::core::Component>::init();
                SpeedLimiterService {
                    _grust_reserved_init_instant,
                    context,
                    delayed,
                    input_store,
                    process_set_speed,
                    speed_limiter,
                    output,
                }
            }
            pub async fn handle_init(
                &mut self,
                speed: f64,
                vacuum_brake: VacuumBrakeState,
                set_speed: f64,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                let _grust_reserved_instant = self._grust_reserved_init_instant;
                self.context.set_speed.set(set_speed);
                self.context.x.set(set_speed);
                self.context.changed_set_speed_old.set(self.context.x.get());
                let ProcessSetSpeedOutput {
                    v_set: v_set_aux,
                    v_update: v_update,
                } = <ProcessSetSpeedState as grust::core::Component>::step(
                    &mut self.process_set_speed,
                    ProcessSetSpeedInput { set_speed: None },
                );
                self.context.v_set_aux.set(v_set_aux);
                self.context.v_update.set(v_update);
                let v_set = self.context.v_set_aux.get();
                self.context.v_set.set(v_set);
                self.send_output(
                    O::VSet(v_set, _grust_reserved_instant),
                    _grust_reserved_instant,
                )
                .await?;
                self.context.vacuum_brake.set(vacuum_brake);
                self.context.speed.set(speed);
                let SpeedLimiterOutput {
                    state: state,
                    on_state: on_state,
                    in_regulation: in_regulation_aux,
                    state_update: state_update,
                } = <SpeedLimiterState as grust::core::Component>::step(
                    &mut self.speed_limiter,
                    SpeedLimiterInput {
                        activation_req: None,
                        vacuum_brake_state: vacuum_brake,
                        kickdown: None,
                        failure: None,
                        speed: speed,
                        v_set: v_set,
                    },
                );
                self.context.state.set(state);
                self.context.on_state.set(on_state);
                self.context.in_regulation_aux.set(in_regulation_aux);
                self.context.state_update.set(state_update);
                self.context
                    .in_regulation_old
                    .set(self.context.in_regulation_aux.get());
                Ok(())
            }
            pub async fn handle_failure(
                &mut self,
                _failure_instant: std::time::Instant,
                failure: Failure,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_failure_instant).await?;
                    self.context.reset();
                    let failure_ref = &mut None;
                    let in_regulation_ref = &mut None;
                    *failure_ref = Some(failure);
                    if failure_ref.is_some()
                        || self.context.vacuum_brake.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let SpeedLimiterOutput {
                            state: state,
                            on_state: on_state,
                            in_regulation: in_regulation_aux,
                            state_update: state_update,
                        } = <SpeedLimiterState as grust::core::Component>::step(
                            &mut self.speed_limiter,
                            SpeedLimiterInput {
                                activation_req: None,
                                vacuum_brake_state: self.context.vacuum_brake.get(),
                                kickdown: None,
                                failure: *failure_ref,
                                speed: self.context.speed.get(),
                                v_set: self.context.v_set.get(),
                            },
                        );
                        self.context.state.set(state);
                        self.context.on_state.set(on_state);
                        self.context.in_regulation_aux.set(in_regulation_aux);
                        self.context.state_update.set(state_update);
                    }
                    if self.context.in_regulation_old.get() != self.context.in_regulation_aux.get()
                    {
                        self.context
                            .in_regulation_old
                            .set(self.context.in_regulation_aux.get());
                        *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                    }
                    if let Some(in_regulation) = *in_regulation_ref {
                        self.send_output(
                            O::InRegulation(in_regulation, _failure_instant),
                            _failure_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .failure
                        .replace((failure, _failure_instant));
                    assert ! (unique . is_none () , "flow `failure` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_speed(
                &mut self,
                _speed_instant: std::time::Instant,
                speed: f64,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_speed_instant).await?;
                    self.context.reset();
                    let in_regulation_ref = &mut None;
                    self.context.speed.set(speed);
                    if self.context.vacuum_brake.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let SpeedLimiterOutput {
                            state: state,
                            on_state: on_state,
                            in_regulation: in_regulation_aux,
                            state_update: state_update,
                        } = <SpeedLimiterState as grust::core::Component>::step(
                            &mut self.speed_limiter,
                            SpeedLimiterInput {
                                activation_req: None,
                                vacuum_brake_state: self.context.vacuum_brake.get(),
                                kickdown: None,
                                failure: None,
                                speed: speed,
                                v_set: self.context.v_set.get(),
                            },
                        );
                        self.context.state.set(state);
                        self.context.on_state.set(on_state);
                        self.context.in_regulation_aux.set(in_regulation_aux);
                        self.context.state_update.set(state_update);
                    }
                    if self.context.in_regulation_old.get() != self.context.in_regulation_aux.get()
                    {
                        self.context
                            .in_regulation_old
                            .set(self.context.in_regulation_aux.get());
                        *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                    }
                    if let Some(in_regulation) = *in_regulation_ref {
                        self.send_output(
                            O::InRegulation(in_regulation, _speed_instant),
                            _speed_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self.input_store.speed.replace((speed, _speed_instant));
                    assert ! (unique . is_none () , "flow `speed` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_vacuum_brake(
                &mut self,
                _vacuum_brake_instant: std::time::Instant,
                vacuum_brake: VacuumBrakeState,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_vacuum_brake_instant).await?;
                    self.context.reset();
                    let in_regulation_ref = &mut None;
                    self.context.vacuum_brake.set(vacuum_brake);
                    if self.context.vacuum_brake.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let SpeedLimiterOutput {
                            state: state,
                            on_state: on_state,
                            in_regulation: in_regulation_aux,
                            state_update: state_update,
                        } = <SpeedLimiterState as grust::core::Component>::step(
                            &mut self.speed_limiter,
                            SpeedLimiterInput {
                                activation_req: None,
                                vacuum_brake_state: vacuum_brake,
                                kickdown: None,
                                failure: None,
                                speed: self.context.speed.get(),
                                v_set: self.context.v_set.get(),
                            },
                        );
                        self.context.state.set(state);
                        self.context.on_state.set(on_state);
                        self.context.in_regulation_aux.set(in_regulation_aux);
                        self.context.state_update.set(state_update);
                    }
                    if self.context.in_regulation_old.get() != self.context.in_regulation_aux.get()
                    {
                        self.context
                            .in_regulation_old
                            .set(self.context.in_regulation_aux.get());
                        *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                    }
                    if let Some(in_regulation) = *in_regulation_ref {
                        self.send_output(
                            O::InRegulation(in_regulation, _vacuum_brake_instant),
                            _vacuum_brake_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .vacuum_brake
                        .replace((vacuum_brake, _vacuum_brake_instant));
                    assert ! (unique . is_none () , "flow `vacuum_brake` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_activation(
                &mut self,
                _activation_instant: std::time::Instant,
                activation: ActivationRequest,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_activation_instant).await?;
                    self.context.reset();
                    let in_regulation_ref = &mut None;
                    let activation_ref = &mut None;
                    *activation_ref = Some(activation);
                    if activation_ref.is_some()
                        || self.context.vacuum_brake.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let SpeedLimiterOutput {
                            state: state,
                            on_state: on_state,
                            in_regulation: in_regulation_aux,
                            state_update: state_update,
                        } = <SpeedLimiterState as grust::core::Component>::step(
                            &mut self.speed_limiter,
                            SpeedLimiterInput {
                                activation_req: *activation_ref,
                                vacuum_brake_state: self.context.vacuum_brake.get(),
                                kickdown: None,
                                failure: None,
                                speed: self.context.speed.get(),
                                v_set: self.context.v_set.get(),
                            },
                        );
                        self.context.state.set(state);
                        self.context.on_state.set(on_state);
                        self.context.in_regulation_aux.set(in_regulation_aux);
                        self.context.state_update.set(state_update);
                    }
                    if self.context.in_regulation_old.get() != self.context.in_regulation_aux.get()
                    {
                        self.context
                            .in_regulation_old
                            .set(self.context.in_regulation_aux.get());
                        *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                    }
                    if let Some(in_regulation) = *in_regulation_ref {
                        self.send_output(
                            O::InRegulation(in_regulation, _activation_instant),
                            _activation_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .activation
                        .replace((activation, _activation_instant));
                    assert ! (unique . is_none () , "flow `activation` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_kickdown(
                &mut self,
                _kickdown_instant: std::time::Instant,
                kickdown: Kickdown,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_kickdown_instant).await?;
                    self.context.reset();
                    let in_regulation_ref = &mut None;
                    let kickdown_ref = &mut None;
                    *kickdown_ref = Some(kickdown);
                    if kickdown_ref.is_some()
                        || self.context.vacuum_brake.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let SpeedLimiterOutput {
                            state: state,
                            on_state: on_state,
                            in_regulation: in_regulation_aux,
                            state_update: state_update,
                        } = <SpeedLimiterState as grust::core::Component>::step(
                            &mut self.speed_limiter,
                            SpeedLimiterInput {
                                activation_req: None,
                                vacuum_brake_state: self.context.vacuum_brake.get(),
                                kickdown: *kickdown_ref,
                                failure: None,
                                speed: self.context.speed.get(),
                                v_set: self.context.v_set.get(),
                            },
                        );
                        self.context.state.set(state);
                        self.context.on_state.set(on_state);
                        self.context.in_regulation_aux.set(in_regulation_aux);
                        self.context.state_update.set(state_update);
                    }
                    if self.context.in_regulation_old.get() != self.context.in_regulation_aux.get()
                    {
                        self.context
                            .in_regulation_old
                            .set(self.context.in_regulation_aux.get());
                        *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                    }
                    if let Some(in_regulation) = *in_regulation_ref {
                        self.send_output(
                            O::InRegulation(in_regulation, _kickdown_instant),
                            _kickdown_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .kickdown
                        .replace((kickdown, _kickdown_instant));
                    assert ! (unique . is_none () , "flow `kickdown` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_set_speed(
                &mut self,
                _set_speed_instant: std::time::Instant,
                set_speed: f64,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_set_speed_instant).await?;
                    self.context.reset();
                    let in_regulation_ref = &mut None;
                    let changed_set_speed_ref = &mut None;
                    self.context.set_speed.set(set_speed);
                    if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                        self.context.x.set(set_speed);
                    }
                    if self.context.changed_set_speed_old.get() != self.context.x.get() {
                        self.context.changed_set_speed_old.set(self.context.x.get());
                        *changed_set_speed_ref = Some(self.context.x.get());
                    }
                    if changed_set_speed_ref.is_some() {
                        let ProcessSetSpeedOutput {
                            v_set: v_set_aux,
                            v_update: v_update,
                        } = <ProcessSetSpeedState as grust::core::Component>::step(
                            &mut self.process_set_speed,
                            ProcessSetSpeedInput {
                                set_speed: *changed_set_speed_ref,
                            },
                        );
                        self.context.v_set_aux.set(v_set_aux);
                        self.context.v_update.set(v_update);
                    }
                    let v_set = self.context.v_set_aux.get();
                    self.context.v_set.set(v_set);
                    if self.context.v_set.is_new() {
                        self.send_output(O::VSet(v_set, _set_speed_instant), _set_speed_instant)
                            .await?;
                    }
                    if self.context.vacuum_brake.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let SpeedLimiterOutput {
                            state: state,
                            on_state: on_state,
                            in_regulation: in_regulation_aux,
                            state_update: state_update,
                        } = <SpeedLimiterState as grust::core::Component>::step(
                            &mut self.speed_limiter,
                            SpeedLimiterInput {
                                activation_req: None,
                                vacuum_brake_state: self.context.vacuum_brake.get(),
                                kickdown: None,
                                failure: None,
                                speed: self.context.speed.get(),
                                v_set: v_set,
                            },
                        );
                        self.context.state.set(state);
                        self.context.on_state.set(on_state);
                        self.context.in_regulation_aux.set(in_regulation_aux);
                        self.context.state_update.set(state_update);
                    }
                    if self.context.in_regulation_old.get() != self.context.in_regulation_aux.get()
                    {
                        self.context
                            .in_regulation_old
                            .set(self.context.in_regulation_aux.get());
                        *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                    }
                    if let Some(in_regulation) = *in_regulation_ref {
                        self.send_output(
                            O::InRegulation(in_regulation, _set_speed_instant),
                            _set_speed_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .set_speed
                        .replace((set_speed, _set_speed_instant));
                    assert ! (unique . is_none () , "flow `set_speed` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            #[inline]
            pub async fn reset_time_constraints(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                Ok(())
            }
            #[inline]
            pub async fn send_output(
                &mut self,
                output: O,
                instant: std::time::Instant,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                self.output.feed(output).await?;
                Ok(())
            }
        }
    }
}
use grust::futures::{Stream, StreamExt};
pub fn run(
    _grust_reserved_init_instant: std::time::Instant,
    input_stream: impl Stream<Item = runtime::RuntimeInput> + Send + 'static,
    init_signals: runtime::RuntimeInit,
) -> grust::futures::channel::mpsc::Receiver<runtime::RuntimeOutput> {
    const OUTPUT_CHANNEL_SIZE: usize = 2usize;
    let (output_sink, output_stream) = grust::futures::channel::mpsc::channel(OUTPUT_CHANNEL_SIZE);
    const PRIO_STREAM_SIZE: usize = 7usize;
    let prio_stream = grust::core::priority_stream::prio_stream::<_, _, PRIO_STREAM_SIZE>(
        input_stream,
        runtime::RuntimeInput::order,
    );
    let service = runtime::Runtime::new(_grust_reserved_init_instant, output_sink);
    grust::tokio::spawn(async move {
        let result = service.run_loop(prio_stream, init_signals).await;
        assert!(result.is_ok())
    });
    output_stream
}
