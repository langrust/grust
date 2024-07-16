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
    let activation_threshold = v_set * 0.99;
    let deactivation_threshold = v_set * 0.98;
    let flag = if prev_hyst.flag && speed <= deactivation_threshold {
        false
    } else {
        if !prev_hyst.flag && speed >= activation_threshold {
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
    let ceiled_speed = if set_speed > 150.0 { 150.0 } else { set_speed };
    let grounded_speed = if set_speed < 10.0 { 10.0 } else { ceiled_speed };
    grounded_speed
}
pub fn off_condition(activation_req: ActivationRequest, vdc_disabled: VdcState) -> bool {
    activation_req == ActivationRequest::Off || vdc_disabled == VdcState::Off
}
pub fn on_condition(activation_req: ActivationRequest) -> bool {
    activation_req == ActivationRequest::On || activation_req == ActivationRequest::Initialization
}
pub fn activation_condition(
    activation_req: ActivationRequest,
    vacuum_brake_state: VacuumBrakeState,
    v_set: f64,
) -> bool {
    activation_req == ActivationRequest::On
        && vacuum_brake_state != VacuumBrakeState::BelowMinLevel
        && v_set > 0.0
}
pub fn exit_override_condition(
    activation_req: ActivationRequest,
    kickdown: KickdownState,
    v_set: f64,
    speed: f64,
) -> bool {
    on_condition(activation_req) && kickdown != KickdownState::Activated && speed <= v_set
}
pub fn involuntary_override_condition(
    activation_req: ActivationRequest,
    kickdown: KickdownState,
    v_set: f64,
    speed: f64,
) -> bool {
    on_condition(activation_req) && kickdown != KickdownState::Activated && speed > v_set
}
pub fn voluntary_override_condition(
    activation_req: ActivationRequest,
    kickdown: KickdownState,
) -> bool {
    on_condition(activation_req) && kickdown == KickdownState::Activated
}
pub fn standby_condition(
    activation_req: ActivationRequest,
    vacuum_brake_state: VacuumBrakeState,
    v_set: f64,
) -> bool {
    activation_req == ActivationRequest::StandBy
        || vacuum_brake_state == VacuumBrakeState::BelowMinLevel
        || v_set <= 0.0
}
pub struct ProcessSetSpeedInput {
    pub set_speed: f64,
}
pub struct ProcessSetSpeedState {
    mem: f64,
}
impl ProcessSetSpeedState {
    pub fn init() -> ProcessSetSpeedState {
        ProcessSetSpeedState { mem: 0.0 }
    }
    pub fn step(&mut self, input: ProcessSetSpeedInput) -> (f64, bool) {
        let v_set = threshold_set_speed(input.set_speed);
        let prev_v_set = self.mem;
        let v_update = prev_v_set != v_set;
        self.mem = v_set;
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
    mem: Hysterisis,
}
impl SpeedLimiterOnState {
    pub fn init() -> SpeedLimiterOnState {
        SpeedLimiterOnState {
            mem: new_hysterisis(0.0),
        }
    }
    pub fn step(&mut self, input: SpeedLimiterOnInput) -> (SpeedLimiterOn, bool) {
        let prev_hysterisis = self.mem;
        let (on_state, hysterisis) = match input.prev_on_state {
            SpeedLimiterOn::StandBy
                if activation_condition(
                    input.activation_req,
                    input.vacuum_brake_state,
                    input.v_set,
                ) =>
            {
                let hysterisis = new_hysterisis(0.0);
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
                let hysterisis = new_hysterisis(0.0);
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
                let hysterisis = new_hysterisis(0.0);
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
                let hysterisis = prev_hysterisis;
                let on_state = input.prev_on_state;
                (on_state, hysterisis)
            }
        };
        let in_reg = in_regulation(hysterisis);
        self.mem = hysterisis;
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
    mem: SpeedLimiter,
    mem_1: SpeedLimiterOn,
    mem_2: bool,
    speed_limiter_on: SpeedLimiterOnState,
}
impl SpeedLimiterState {
    pub fn init() -> SpeedLimiterState {
        SpeedLimiterState {
            mem: SpeedLimiter::Off,
            mem_1: SpeedLimiterOn::StandBy,
            mem_2: false,
            speed_limiter_on: SpeedLimiterOnState::init(),
        }
    }
    pub fn step(&mut self, input: SpeedLimiterInput) -> (SpeedLimiter, SpeedLimiterOn, bool, bool) {
        let failure = false;
        let prev_state = self.mem;
        let prev_on_state = self.mem_1;
        let (state, on_state, in_regulation) = match prev_state {
            _ if off_condition(input.activation_req, input.vdc_disabled) => {
                let state = SpeedLimiter::Off;
                let on_state = prev_on_state;
                let in_regulation = false;
                (state, on_state, in_regulation)
            }
            SpeedLimiter::Off if on_condition(input.activation_req) => {
                let (state, on_state, in_regulation) = match failure {
                    true => {
                        let state = SpeedLimiter::Fail;
                        let on_state = prev_on_state;
                        let in_regulation = false;
                        (state, on_state, in_regulation)
                    }
                    false => {
                        let state = SpeedLimiter::On;
                        let on_state = SpeedLimiterOn::StandBy;
                        let in_regulation = true;
                        (state, on_state, in_regulation)
                    }
                };
                (state, on_state, in_regulation)
            }
            SpeedLimiter::On if failure => {
                let state = SpeedLimiter::Fail;
                let on_state = prev_on_state;
                let in_regulation = false;
                (state, on_state, in_regulation)
            }
            SpeedLimiter::Fail if !failure => {
                let state = SpeedLimiter::On;
                let on_state = SpeedLimiterOn::StandBy;
                let in_regulation = true;
                (state, on_state, in_regulation)
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
                (state, on_state, in_regulation)
            }
            _ => {
                let state = prev_state;
                let on_state = prev_on_state;
                let in_regulation = self.mem_2;
                (state, on_state, in_regulation)
            }
        };
        let state_update = state != prev_state || on_state != prev_on_state;
        self.mem = state;
        self.mem_1 = on_state;
        self.mem_2 = in_regulation;
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
    }
    impl timer_stream::Timing for RuntimeTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::PeriodSpeedLimiter => std::time::Duration::from_millis(10u64),
                T::DelaySpeedLimiter => std::time::Duration::from_millis(10u64),
                T::TimeoutSpeedLimiter => std::time::Duration::from_millis(500u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::PeriodSpeedLimiter => false,
                T::DelaySpeedLimiter => true,
                T::TimeoutSpeedLimiter => true,
            }
        }
    }
    pub enum RuntimeInput {
        SetSpeed(f64, std::time::Instant),
        VacuumBrake(VacuumBrakeState, std::time::Instant),
        Activation(ActivationRequest, std::time::Instant),
        Vdc(VdcState, std::time::Instant),
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
                (I::SetSpeed(this, _), I::SetSpeed(other, _)) => this.eq(other),
                (I::VacuumBrake(this, _), I::VacuumBrake(other, _)) => this.eq(other),
                (I::Activation(this, _), I::Activation(other, _)) => this.eq(other),
                (I::Vdc(this, _), I::Vdc(other, _)) => this.eq(other),
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
                I::SetSpeed(_, instant) => *instant,
                I::VacuumBrake(_, instant) => *instant,
                I::Activation(_, instant) => *instant,
                I::Vdc(_, instant) => *instant,
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
        VSet(f64, std::time::Instant),
        InRegulation(bool, std::time::Instant),
    }
    pub struct Runtime {
        speed_limiter: speed_limiter_service::SpeedLimiterService,
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl Runtime {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let speed_limiter =
                speed_limiter_service::SpeedLimiterService::init(output, timer.clone());
            Runtime {
                speed_limiter,
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
                .send_timer(T::PeriodSpeedLimiter, init_instant)
                .await?;
            runtime
                .send_timer(T::TimeoutSpeedLimiter, init_instant)
                .await?;
            while let Some(input) = input.next().await {
                match input {
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
                    }
                    I::Vdc(vdc, instant) => {
                        runtime.speed_limiter.handle_vdc(instant, vdc).await?;
                    }
                    I::Timer(T::TimeoutSpeedLimiter, instant) => {
                        runtime
                            .speed_limiter
                            .handle_timeout_speed_limiter(instant)
                            .await?;
                    }
                    I::Speed(speed, instant) => {
                        runtime.speed_limiter.handle_speed(instant, speed).await?;
                    }
                    I::Kickdown(kickdown, instant) => {
                        runtime
                            .speed_limiter
                            .handle_kickdown(instant, kickdown)
                            .await?;
                    }
                    I::Timer(T::DelaySpeedLimiter, instant) => {
                        runtime
                            .speed_limiter
                            .handle_delay_speed_limiter(instant)
                            .await?;
                    }
                    I::SetSpeed(set_speed, instant) => {
                        runtime
                            .speed_limiter
                            .handle_set_speed(instant, set_speed)
                            .await?;
                    }
                    I::VacuumBrake(vacuum_brake, instant) => {
                        runtime
                            .speed_limiter
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
        pub struct Context {
            pub state_update: bool,
            pub v_update: bool,
            pub activation: ActivationRequest,
            pub kickdown: KickdownState,
            pub vdc: VdcState,
            pub set_speed: f64,
            pub v_set_aux: f64,
            pub speed: f64,
            pub v_set: f64,
            pub in_regulation_aux: bool,
            pub vacuum_brake: VacuumBrakeState,
            pub on_state: SpeedLimiterOn,
            pub state: SpeedLimiter,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn get_process_set_speed_inputs(&self) -> ProcessSetSpeedInput {
                ProcessSetSpeedInput {
                    set_speed: self.set_speed,
                }
            }
            fn get_speed_limiter_inputs(&self) -> SpeedLimiterInput {
                SpeedLimiterInput {
                    activation_req: self.activation,
                    vacuum_brake_state: self.vacuum_brake,
                    kickdown: self.kickdown,
                    vdc_disabled: self.vdc,
                    speed: self.speed,
                    v_set: self.v_set,
                }
            }
        }
        #[derive(Default)]
        pub struct SpeedLimiterServiceStore {
            period_speed_limiter: Option<((), std::time::Instant)>,
            activation: Option<(ActivationRequest, std::time::Instant)>,
            vdc: Option<(VdcState, std::time::Instant)>,
            speed: Option<(f64, std::time::Instant)>,
            kickdown: Option<(KickdownState, std::time::Instant)>,
            set_speed: Option<(f64, std::time::Instant)>,
            vacuum_brake: Option<(VacuumBrakeState, std::time::Instant)>,
        }
        impl SpeedLimiterServiceStore {
            pub fn not_empty(&self) -> bool {
                self.period_speed_limiter.is_some()
                    || self.activation.is_some()
                    || self.vdc.is_some()
                    || self.speed.is_some()
                    || self.kickdown.is_some()
                    || self.set_speed.is_some()
                    || self.vacuum_brake.is_some()
            }
        }
        pub struct SpeedLimiterService {
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
                    context,
                    delayed,
                    input_store,
                    process_set_speed,
                    speed_limiter,
                    output,
                    timer,
                }
            }
            pub async fn handle_period_speed_limiter(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(instant).await?;
                    let (v_set_aux, v_update) = self
                        .process_set_speed
                        .step(self.context.get_process_set_speed_inputs());
                    self.context.v_set_aux = v_set_aux;
                    self.context.v_update = v_update;
                    let v_set_aux = self.context.v_set_aux;
                    let v_set = v_set_aux;
                    self.context.v_set = v_set;
                    self.send_output(O::VSet(v_set, instant)).await?;
                    let (state, on_state, in_regulation_aux, state_update) = self
                        .speed_limiter
                        .step(self.context.get_speed_limiter_inputs());
                    self.context.state = state;
                    self.context.on_state = on_state;
                    self.context.in_regulation_aux = in_regulation_aux;
                    self.context.state_update = state_update;
                    let in_regulation_aux = self.context.in_regulation_aux;
                    let in_regulation = in_regulation_aux;
                    self.send_output(O::InRegulation(in_regulation, instant))
                        .await?;
                    self.send_timer(T::PeriodSpeedLimiter, instant).await?;
                } else {
                    let unique = self.input_store.period_speed_limiter.replace(((), instant));
                    assert!(
                        unique.is_none(),
                        "period_speed_limiter changes too frequently"
                    );
                }
                Ok(())
            }
            pub async fn handle_activation(
                &mut self,
                instant: std::time::Instant,
                activation: ActivationRequest,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(instant).await?;
                    self.context.activation = activation;
                } else {
                    let unique = self.input_store.activation.replace((activation, instant));
                    assert!(unique.is_none(), "activation changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_vdc(
                &mut self,
                instant: std::time::Instant,
                vdc: VdcState,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(instant).await?;
                    self.context.vdc = vdc;
                } else {
                    let unique = self.input_store.vdc.replace((vdc, instant));
                    assert!(unique.is_none(), "vdc changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_timeout_speed_limiter(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constrains(instant).await?;
                let (v_set_aux, v_update) = self
                    .process_set_speed
                    .step(self.context.get_process_set_speed_inputs());
                self.context.v_set_aux = v_set_aux;
                self.context.v_update = v_update;
                let v_set_aux = self.context.v_set_aux;
                let v_set = v_set_aux;
                self.context.v_set = v_set;
                self.send_output(O::VSet(v_set, instant)).await?;
                let (state, on_state, in_regulation_aux, state_update) = self
                    .speed_limiter
                    .step(self.context.get_speed_limiter_inputs());
                self.context.state = state;
                self.context.on_state = on_state;
                self.context.in_regulation_aux = in_regulation_aux;
                self.context.state_update = state_update;
                let in_regulation_aux = self.context.in_regulation_aux;
                let in_regulation = in_regulation_aux;
                self.send_output(O::InRegulation(in_regulation, instant))
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
            pub async fn handle_speed(
                &mut self,
                instant: std::time::Instant,
                speed: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(instant).await?;
                    self.context.speed = speed;
                } else {
                    let unique = self.input_store.speed.replace((speed, instant));
                    assert!(unique.is_none(), "speed changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_kickdown(
                &mut self,
                instant: std::time::Instant,
                kickdown: KickdownState,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(instant).await?;
                    self.context.kickdown = kickdown;
                } else {
                    let unique = self.input_store.kickdown.replace((kickdown, instant));
                    assert!(unique.is_none(), "kickdown changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_delay_speed_limiter(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.input_store.not_empty() {
                    self.reset_time_constrains(instant).await?;
                    self.handle_input_store(instant).await?;
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
            pub async fn handle_set_speed(
                &mut self,
                instant: std::time::Instant,
                set_speed: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(instant).await?;
                    self.context.set_speed = set_speed;
                } else {
                    let unique = self.input_store.set_speed.replace((set_speed, instant));
                    assert!(unique.is_none(), "set_speed changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_vacuum_brake(
                &mut self,
                instant: std::time::Instant,
                vacuum_brake: VacuumBrakeState,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(instant).await?;
                    self.context.vacuum_brake = vacuum_brake;
                } else {
                    let unique = self
                        .input_store
                        .vacuum_brake
                        .replace((vacuum_brake, instant));
                    assert!(unique.is_none(), "vacuum_brake changes too frequently");
                }
                Ok(())
            }
            #[inline]
            pub async fn handle_input_store(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                todo!();
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
