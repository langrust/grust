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
    Actif,
    OverrideVoluntary,
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
pub fn activation_condition(vacuum_brake_state: VacuumBrakeState, v_set: f64) -> bool {
    (vacuum_brake_state != VacuumBrakeState::BelowMinLevel) && (v_set > 0.0f64)
}
pub fn standby_condition(vacuum_brake_state: VacuumBrakeState, v_set: f64) -> bool {
    (vacuum_brake_state == VacuumBrakeState::BelowMinLevel) || (v_set <= 0.0f64)
}
pub struct ProcessSetSpeedInput {
    pub set_speed: Option<f64>,
}
pub struct ProcessSetSpeedState {
    last_v_set: f64,
}
impl ProcessSetSpeedState {
    pub fn init() -> ProcessSetSpeedState {
        ProcessSetSpeedState { last_v_set: 0.0f64 }
    }
    pub fn step(&mut self, input: ProcessSetSpeedInput) -> (f64, bool) {
        let prev_v_set = self.last_v_set;
        let v_set = match (input.set_speed) {
            (Some(v)) => threshold_set_speed(v),
            (_) => self.last_v_set,
        };
        let v_update = prev_v_set != v_set;
        self.last_v_set = v_set;
        (v_set, v_update)
    }
}
pub struct SpeedLimiterOnInput {
    pub prev_on_state: SpeedLimiterOn,
    pub vacuum_brake_state: VacuumBrakeState,
    pub kickdown: Option<Kickdown>,
    pub speed: f64,
    pub v_set: f64,
}
pub struct SpeedLimiterOnState {
    last_hysterisis: Hysterisis,
    last_kickdown_state: Kickdown,
}
impl SpeedLimiterOnState {
    pub fn init() -> SpeedLimiterOnState {
        SpeedLimiterOnState {
            last_hysterisis: new_hysterisis(0.0f64),
            last_kickdown_state: Kickdown::Deactivated,
        }
    }
    pub fn step(&mut self, input: SpeedLimiterOnInput) -> (SpeedLimiterOn, bool, bool) {
        let prev_hysterisis = self.last_hysterisis;
        let kickdown_state = match (input.kickdown) {
            (Some(Kickdown::Activated)) if input.prev_on_state == SpeedLimiterOn::Actif => {
                Kickdown::Activated
            }
            (Some(Kickdown::Deactivated)) => Kickdown::Deactivated,
            (_) => self.last_kickdown_state,
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
                let on_state = SpeedLimiterOn::Actif;
                let hysterisis = new_hysterisis(0.0f64);
                (hysterisis, on_state)
            }
            SpeedLimiterOn::OverrideVoluntary if input.speed <= input.v_set => {
                let on_state = SpeedLimiterOn::Actif;
                let hysterisis = new_hysterisis(0.0f64);
                (hysterisis, on_state)
            }
            SpeedLimiterOn::Actif if standby_condition(input.vacuum_brake_state, input.v_set) => {
                let on_state = SpeedLimiterOn::StandBy;
                let hysterisis = prev_hysterisis;
                (hysterisis, on_state)
            }
            SpeedLimiterOn::Actif => {
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
        (on_state, in_reg, state_update)
    }
}
pub struct SpeedLimiterInput {
    pub activation_req: Option<ActivationRequest>,
    pub vacuum_brake_state: VacuumBrakeState,
    pub kickdown: Option<Kickdown>,
    pub failure: Option<Failure>,
    pub vdc_disabled: VdcState,
    pub speed: f64,
    pub v_set: f64,
}
pub struct SpeedLimiterState {
    last_on_state: SpeedLimiterOn,
    last_state: SpeedLimiter,
    speed_limiter_on: SpeedLimiterOnState,
}
impl SpeedLimiterState {
    pub fn init() -> SpeedLimiterState {
        SpeedLimiterState {
            last_on_state: SpeedLimiterOn::StandBy,
            last_state: SpeedLimiter::Off,
            speed_limiter_on: SpeedLimiterOnState::init(),
        }
    }
    pub fn step(&mut self, input: SpeedLimiterInput) -> (SpeedLimiter, SpeedLimiterOn, bool, bool) {
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
            (_, _) => self.last_state,
        };
        let (state_update, on_state, in_regulation) = match prev_state {
            SpeedLimiter::On => {
                let (on_state, in_regulation, state_update) =
                    self.speed_limiter_on.step(SpeedLimiterOnInput {
                        prev_on_state: prev_on_state,
                        vacuum_brake_state: input.vacuum_brake_state,
                        kickdown: input.kickdown,
                        speed: input.speed,
                        v_set: input.v_set,
                    });
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
        DelaySpeedLimiter,
        TimeoutSpeedLimiter,
    }
    impl timer_stream::Timing for RuntimeTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::DelaySpeedLimiter => std::time::Duration::from_millis(10u64),
                T::TimeoutSpeedLimiter => std::time::Duration::from_millis(500u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::DelaySpeedLimiter => true,
                T::TimeoutSpeedLimiter => true,
            }
        }
    }
    pub enum RuntimeInput {
        Vdc(VdcState, std::time::Instant),
        VacuumBrake(VacuumBrakeState, std::time::Instant),
        Activation(ActivationRequest, std::time::Instant),
        Kickdown(Kickdown, std::time::Instant),
        SetSpeed(f64, std::time::Instant),
        Failure(Failure, std::time::Instant),
        Speed(f64, std::time::Instant),
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
                (I::Vdc(this, _), I::Vdc(other, _)) => this.eq(other),
                (I::VacuumBrake(this, _), I::VacuumBrake(other, _)) => this.eq(other),
                (I::Activation(this, _), I::Activation(other, _)) => this.eq(other),
                (I::Kickdown(this, _), I::Kickdown(other, _)) => this.eq(other),
                (I::SetSpeed(this, _), I::SetSpeed(other, _)) => this.eq(other),
                (I::Failure(this, _), I::Failure(other, _)) => this.eq(other),
                (I::Speed(this, _), I::Speed(other, _)) => this.eq(other),
                (I::Timer(this, _), I::Timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::Vdc(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::VacuumBrake(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Activation(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Kickdown(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::SetSpeed(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Failure(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Speed(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Timer(_, _grust_reserved_instant) => *_grust_reserved_instant,
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
        output: futures::channel::mpsc::Sender<O>,
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl Runtime {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let speed_limiter =
                speed_limiter_service::SpeedLimiterService::init(output.clone(), timer.clone());
            Runtime {
                speed_limiter,
                output,
                timer,
            }
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
        pub async fn run_loop(
            self,
            _grust_reserved_init_instant: std::time::Instant,
            input: impl futures::Stream<Item = I>,
        ) -> Result<(), futures::channel::mpsc::SendError> {
            futures::pin_mut!(input);
            let mut runtime = self;
            runtime
                .send_timer(T::TimeoutSpeedLimiter, _grust_reserved_init_instant)
                .await?;
            runtime
                .send_output(O::VSet(Default::default(), _grust_reserved_init_instant))
                .await?;
            while let Some(input) = input.next().await {
                match input {
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
                    I::Speed(speed, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_speed(_grust_reserved_instant, speed)
                            .await?;
                    }
                    I::Activation(activation, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_activation(_grust_reserved_instant, activation)
                            .await?;
                    }
                    I::Timer(T::DelaySpeedLimiter, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_delay_speed_limiter(_grust_reserved_instant)
                            .await?;
                    }
                    I::VacuumBrake(vacuum_brake, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_vacuum_brake(_grust_reserved_instant, vacuum_brake)
                            .await?;
                    }
                    I::Failure(failure, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_failure(_grust_reserved_instant, failure)
                            .await?;
                    }
                    I::Timer(T::TimeoutSpeedLimiter, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_timeout_speed_limiter(_grust_reserved_instant)
                            .await?;
                    }
                    I::Vdc(vdc, _grust_reserved_instant) => {
                        runtime
                            .speed_limiter
                            .handle_vdc(_grust_reserved_instant, vdc)
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
        mod ctx_ty {
            use super::*;
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
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct VacuumBrake(VacuumBrakeState, bool);
            impl VacuumBrake {
                pub fn set(&mut self, vacuum_brake: VacuumBrakeState) {
                    self.1 = self.0 != vacuum_brake;
                    self.0 = vacuum_brake;
                }
                pub fn get(&self) -> VacuumBrakeState {
                    self.0
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct Vdc(VdcState, bool);
            impl Vdc {
                pub fn set(&mut self, vdc: VdcState) {
                    self.1 = self.0 != vdc;
                    self.0 = vdc;
                }
                pub fn get(&self) -> VdcState {
                    self.0
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
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct OnState(SpeedLimiterOn, bool);
            impl OnState {
                pub fn set(&mut self, on_state: SpeedLimiterOn) {
                    self.1 = self.0 != on_state;
                    self.0 = on_state;
                }
                pub fn get(&self) -> SpeedLimiterOn {
                    self.0
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct State(SpeedLimiter, bool);
            impl State {
                pub fn set(&mut self, state: SpeedLimiter) {
                    self.1 = self.0 != state;
                    self.0 = state;
                }
                pub fn get(&self) -> SpeedLimiter {
                    self.0
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
            pub vdc: ctx_ty::Vdc,
            pub speed: ctx_ty::Speed,
            pub changed_set_speed_old: ctx_ty::ChangedSetSpeedOld,
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
                self.vdc.reset();
                self.speed.reset();
                self.changed_set_speed_old.reset();
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
            vdc: Option<(VdcState, std::time::Instant)>,
            vacuum_brake: Option<(VacuumBrakeState, std::time::Instant)>,
            activation: Option<(ActivationRequest, std::time::Instant)>,
            kickdown: Option<(Kickdown, std::time::Instant)>,
            set_speed: Option<(f64, std::time::Instant)>,
            failure: Option<(Failure, std::time::Instant)>,
            speed: Option<(f64, std::time::Instant)>,
        }
        impl SpeedLimiterServiceStore {
            pub fn not_empty(&self) -> bool {
                self.vdc.is_some()
                    || self.vacuum_brake.is_some()
                    || self.activation.is_some()
                    || self.kickdown.is_some()
                    || self.set_speed.is_some()
                    || self.failure.is_some()
                    || self.speed.is_some()
            }
        }
        pub struct SpeedLimiterService {
            begin: std::time::Instant,
            context: Context,
            delayed: bool,
            input_store: SpeedLimiterServiceStore,
            process_set_speed: ProcessSetSpeedState,
            speed_limiter: SpeedLimiterState,
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
                let process_set_speed = ProcessSetSpeedState::init();
                let speed_limiter = SpeedLimiterState::init();
                SpeedLimiterService {
                    begin: std::time::Instant::now(),
                    context,
                    delayed,
                    input_store,
                    process_set_speed,
                    speed_limiter,
                    output,
                    timer,
                }
            }
            pub async fn handle_vdc(
                &mut self,
                _vdc_instant: std::time::Instant,
                vdc: VdcState,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_vdc_instant).await?;
                    self.context.reset();
                    let in_regulation_ref = &mut None;
                    self.context.vdc.set(vdc);
                    if self.context.vacuum_brake.is_new()
                        || self.context.vdc.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let (state, on_state, in_regulation_aux, state_update) =
                            self.speed_limiter.step(SpeedLimiterInput {
                                activation_req: None,
                                vacuum_brake_state: self.context.vacuum_brake.get(),
                                kickdown: None,
                                failure: None,
                                vdc_disabled: vdc,
                                speed: self.context.speed.get(),
                                v_set: self.context.v_set.get(),
                            });
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
                            O::InRegulation(in_regulation, _vdc_instant),
                            _vdc_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self.input_store.vdc.replace((vdc, _vdc_instant));
                    assert!(unique.is_none(), "flow `vdc` changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_vacuum_brake(
                &mut self,
                _vacuum_brake_instant: std::time::Instant,
                vacuum_brake: VacuumBrakeState,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_vacuum_brake_instant).await?;
                    self.context.reset();
                    let in_regulation_ref = &mut None;
                    self.context.vacuum_brake.set(vacuum_brake);
                    if self.context.vacuum_brake.is_new()
                        || self.context.vdc.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let (state, on_state, in_regulation_aux, state_update) =
                            self.speed_limiter.step(SpeedLimiterInput {
                                activation_req: None,
                                vacuum_brake_state: vacuum_brake,
                                kickdown: None,
                                failure: None,
                                vdc_disabled: self.context.vdc.get(),
                                speed: self.context.speed.get(),
                                v_set: self.context.v_set.get(),
                            });
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
                    assert!(
                        unique.is_none(),
                        "flow `vacuum_brake` changes too frequently"
                    );
                }
                Ok(())
            }
            pub async fn handle_activation(
                &mut self,
                _activation_instant: std::time::Instant,
                activation: ActivationRequest,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_activation_instant).await?;
                    self.context.reset();
                    let activation_ref = &mut None;
                    let in_regulation_ref = &mut None;
                    *activation_ref = Some(activation);
                    if activation_ref.is_some()
                        || self.context.vacuum_brake.is_new()
                        || self.context.vdc.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let (state, on_state, in_regulation_aux, state_update) =
                            self.speed_limiter.step(SpeedLimiterInput {
                                activation_req: *activation_ref,
                                vacuum_brake_state: self.context.vacuum_brake.get(),
                                kickdown: None,
                                failure: None,
                                vdc_disabled: self.context.vdc.get(),
                                speed: self.context.speed.get(),
                                v_set: self.context.v_set.get(),
                            });
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
                    assert!(unique.is_none(), "flow `activation` changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_kickdown(
                &mut self,
                _kickdown_instant: std::time::Instant,
                kickdown: Kickdown,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_kickdown_instant).await?;
                    self.context.reset();
                    let kickdown_ref = &mut None;
                    let in_regulation_ref = &mut None;
                    *kickdown_ref = Some(kickdown);
                    if kickdown_ref.is_some()
                        || self.context.vacuum_brake.is_new()
                        || self.context.vdc.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let (state, on_state, in_regulation_aux, state_update) =
                            self.speed_limiter.step(SpeedLimiterInput {
                                activation_req: None,
                                vacuum_brake_state: self.context.vacuum_brake.get(),
                                kickdown: *kickdown_ref,
                                failure: None,
                                vdc_disabled: self.context.vdc.get(),
                                speed: self.context.speed.get(),
                                v_set: self.context.v_set.get(),
                            });
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
                    assert!(unique.is_none(), "flow `kickdown` changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_delay_speed_limiter(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.context.reset();
                if self.input_store.not_empty() {
                    self.reset_time_constraints(_grust_reserved_instant).await?;
                    match (
                        self.input_store.vdc.take(),
                        self.input_store.vacuum_brake.take(),
                        self.input_store.activation.take(),
                        self.input_store.kickdown.take(),
                        self.input_store.set_speed.take(),
                        self.input_store.failure.take(),
                        self.input_store.speed.take(),
                    ) {
                        (None, None, None, None, None, None, None) => {}
                        (Some((vdc, _vdc_instant)), None, None, None, None, None, None) => {
                            let in_regulation_ref = &mut None;
                            self.context.vdc.set(vdc);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            let in_regulation_ref = &mut None;
                            self.context.vacuum_brake.set(vacuum_brake);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            let in_regulation_ref = &mut None;
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            None,
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *kickdown_ref = Some(kickdown);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *kickdown_ref = Some(kickdown);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vdc.set(vdc);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vacuum_brake.set(vacuum_brake);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, None, None, None, None, Some((failure, _failure_instant)), None) => {
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            self.context.vdc.set(vdc);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vdc.set(vdc);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vacuum_brake.set(vacuum_brake);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            None,
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: self.context.speed.get(),
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, None, None, None, None, None, Some((speed, _speed_instant))) => {
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            None,
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            self.context.vdc.set(vdc);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *kickdown_ref = Some(kickdown);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *kickdown_ref = Some(kickdown);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vdc.set(vdc);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vacuum_brake.set(vacuum_brake);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            None,
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: None,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            self.context.vdc.set(vdc);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            None,
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: self.context.v_set.get(),
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vdc.set(vdc);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vacuum_brake.set(vacuum_brake);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            None,
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: None,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            None,
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: None,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            None,
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: self.context.vacuum_brake.get(),
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: self.context.vdc.get(),
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((vdc, _vdc_instant)),
                            Some((vacuum_brake, _vacuum_brake_instant)),
                            Some((activation, _activation_instant)),
                            Some((kickdown, _kickdown_instant)),
                            Some((set_speed, _set_speed_instant)),
                            Some((failure, _failure_instant)),
                            Some((speed, _speed_instant)),
                        ) => {
                            let kickdown_ref = &mut None;
                            let activation_ref = &mut None;
                            let failure_ref = &mut None;
                            let changed_set_speed_ref = &mut None;
                            let in_regulation_ref = &mut None;
                            self.context.speed.set(speed);
                            *failure_ref = Some(failure);
                            if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                                self.context.x.set(set_speed);
                            }
                            if self.context.changed_set_speed_old.get() != self.context.x.get() {
                                self.context.changed_set_speed_old.set(self.context.x.get());
                                *changed_set_speed_ref = Some(self.context.x.get());
                            }
                            if changed_set_speed_ref.is_some() {
                                let (v_set_aux, v_update) =
                                    self.process_set_speed.step(ProcessSetSpeedInput {
                                        set_speed: *changed_set_speed_ref,
                                    });
                                self.context.v_set_aux.set(v_set_aux);
                                self.context.v_update.set(v_update);
                            }
                            let v_set = self.context.v_set_aux.get();
                            self.context.v_set.set(v_set);
                            if self.context.v_set.is_new() {
                                self.send_output(
                                    O::VSet(v_set, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *kickdown_ref = Some(kickdown);
                            *activation_ref = Some(activation);
                            self.context.vacuum_brake.set(vacuum_brake);
                            self.context.vdc.set(vdc);
                            if activation_ref.is_some()
                                || kickdown_ref.is_some()
                                || failure_ref.is_some()
                                || self.context.vacuum_brake.is_new()
                                || self.context.vdc.is_new()
                                || self.context.speed.is_new()
                                || self.context.v_set.is_new()
                            {
                                let (state, on_state, in_regulation_aux, state_update) =
                                    self.speed_limiter.step(SpeedLimiterInput {
                                        activation_req: *activation_ref,
                                        vacuum_brake_state: vacuum_brake,
                                        kickdown: *kickdown_ref,
                                        failure: *failure_ref,
                                        vdc_disabled: vdc,
                                        speed: speed,
                                        v_set: v_set,
                                    });
                                self.context.state.set(state);
                                self.context.on_state.set(on_state);
                                self.context.in_regulation_aux.set(in_regulation_aux);
                                self.context.state_update.set(state_update);
                            }
                            if self.context.in_regulation_old.get()
                                != self.context.in_regulation_aux.get()
                            {
                                self.context
                                    .in_regulation_old
                                    .set(self.context.in_regulation_aux.get());
                                *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                            }
                            if let Some(in_regulation) = *in_regulation_ref {
                                self.send_output(
                                    O::InRegulation(in_regulation, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
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
                _grust_reserved_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer
                    .send((T::DelaySpeedLimiter, _grust_reserved_instant))
                    .await?;
                Ok(())
            }
            pub async fn handle_set_speed(
                &mut self,
                _set_speed_instant: std::time::Instant,
                set_speed: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_set_speed_instant).await?;
                    self.context.reset();
                    let changed_set_speed_ref = &mut None;
                    let in_regulation_ref = &mut None;
                    if (self.context.x.get() - set_speed).abs() >= 1.0f64 {
                        self.context.x.set(set_speed);
                    }
                    if self.context.changed_set_speed_old.get() != self.context.x.get() {
                        self.context.changed_set_speed_old.set(self.context.x.get());
                        *changed_set_speed_ref = Some(self.context.x.get());
                    }
                    if changed_set_speed_ref.is_some() {
                        let (v_set_aux, v_update) =
                            self.process_set_speed.step(ProcessSetSpeedInput {
                                set_speed: *changed_set_speed_ref,
                            });
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
                        || self.context.vdc.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let (state, on_state, in_regulation_aux, state_update) =
                            self.speed_limiter.step(SpeedLimiterInput {
                                activation_req: None,
                                vacuum_brake_state: self.context.vacuum_brake.get(),
                                kickdown: None,
                                failure: None,
                                vdc_disabled: self.context.vdc.get(),
                                speed: self.context.speed.get(),
                                v_set: v_set,
                            });
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
                    assert!(unique.is_none(), "flow `set_speed` changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_failure(
                &mut self,
                _failure_instant: std::time::Instant,
                failure: Failure,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_failure_instant).await?;
                    self.context.reset();
                    let failure_ref = &mut None;
                    let in_regulation_ref = &mut None;
                    *failure_ref = Some(failure);
                    if failure_ref.is_some()
                        || self.context.vacuum_brake.is_new()
                        || self.context.vdc.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let (state, on_state, in_regulation_aux, state_update) =
                            self.speed_limiter.step(SpeedLimiterInput {
                                activation_req: None,
                                vacuum_brake_state: self.context.vacuum_brake.get(),
                                kickdown: None,
                                failure: *failure_ref,
                                vdc_disabled: self.context.vdc.get(),
                                speed: self.context.speed.get(),
                                v_set: self.context.v_set.get(),
                            });
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
                    assert!(unique.is_none(), "flow `failure` changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_timeout_speed_limiter(
                &mut self,
                _timeout_speed_limiter_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constraints(_timeout_speed_limiter_instant)
                    .await?;
                self.context.reset();
                let in_regulation_ref = &mut None;
                let v_set = self.context.v_set_aux.get();
                self.context.v_set.set(v_set);
                self.send_output(
                    O::VSet(v_set, _timeout_speed_limiter_instant),
                    _timeout_speed_limiter_instant,
                )
                .await?;
                if self.context.vacuum_brake.is_new()
                    || self.context.vdc.is_new()
                    || self.context.speed.is_new()
                    || self.context.v_set.is_new()
                {
                    let (state, on_state, in_regulation_aux, state_update) =
                        self.speed_limiter.step(SpeedLimiterInput {
                            activation_req: None,
                            vacuum_brake_state: self.context.vacuum_brake.get(),
                            kickdown: None,
                            failure: None,
                            vdc_disabled: self.context.vdc.get(),
                            speed: self.context.speed.get(),
                            v_set: v_set,
                        });
                    self.context.state.set(state);
                    self.context.on_state.set(on_state);
                    self.context.in_regulation_aux.set(in_regulation_aux);
                    self.context.state_update.set(state_update);
                }
                if self.context.in_regulation_old.get() != self.context.in_regulation_aux.get() {
                    self.context
                        .in_regulation_old
                        .set(self.context.in_regulation_aux.get());
                    *in_regulation_ref = Some(self.context.in_regulation_aux.get());
                }
                if let Some(in_regulation) = *in_regulation_ref {
                    self.send_output(
                        O::InRegulation(in_regulation, _timeout_speed_limiter_instant),
                        _timeout_speed_limiter_instant,
                    )
                    .await?;
                }
                Ok(())
            }
            #[inline]
            pub async fn reset_service_timeout(
                &mut self,
                _timeout_speed_limiter_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer
                    .send((T::TimeoutSpeedLimiter, _timeout_speed_limiter_instant))
                    .await?;
                Ok(())
            }
            pub async fn handle_speed(
                &mut self,
                _speed_instant: std::time::Instant,
                speed: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_speed_instant).await?;
                    self.context.reset();
                    let in_regulation_ref = &mut None;
                    self.context.speed.set(speed);
                    if self.context.vacuum_brake.is_new()
                        || self.context.vdc.is_new()
                        || self.context.speed.is_new()
                        || self.context.v_set.is_new()
                    {
                        let (state, on_state, in_regulation_aux, state_update) =
                            self.speed_limiter.step(SpeedLimiterInput {
                                activation_req: None,
                                vacuum_brake_state: self.context.vacuum_brake.get(),
                                kickdown: None,
                                failure: None,
                                vdc_disabled: self.context.vdc.get(),
                                speed: speed,
                                v_set: self.context.v_set.get(),
                            });
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
                    assert!(unique.is_none(), "flow `speed` changes too frequently");
                }
                Ok(())
            }
            #[inline]
            pub async fn reset_time_constraints(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_delay(instant).await?;
                self.delayed = false;
                Ok(())
            }
            #[inline]
            pub async fn send_output(
                &mut self,
                output: O,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_timeout(instant).await?;
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
