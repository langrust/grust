#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Hysterisis {
    pub value: f64,
    pub flag: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ActivationResquest {
    #[default]
    Off,
    On,
    Initialization,
    StandBy,
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum VdcState {
    #[default]
    On,
    Off,
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum VacuumBrakeState {
    #[default]
    BelowMinLevel,
    AboveMinLevel,
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum KickdownState {
    #[default]
    Deactivated,
    Activated,
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum SpeedLimiter {
    #[default]
    Off,
    On,
    Fail,
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
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
pub fn off_condition(activation_req: ActivationResquest, vdc_disabled: VdcState) -> bool {
    activation_req == ActivationResquest::Off || vdc_disabled == VdcState::Off
}
pub fn on_condition(activation_req: ActivationResquest) -> bool {
    activation_req == ActivationResquest::On || activation_req == ActivationResquest::Initialization
}
pub fn activation_condition(
    activation_req: ActivationResquest,
    vacuum_brake_state: VacuumBrakeState,
    v_set: f64,
) -> bool {
    activation_req == ActivationResquest::On
        && vacuum_brake_state != VacuumBrakeState::BelowMinLevel
        && v_set > 0.0
}
pub fn exit_override_condition(
    activation_req: ActivationResquest,
    kickdown: KickdownState,
    v_set: f64,
    speed: f64,
) -> bool {
    on_condition(activation_req) && kickdown != KickdownState::Activated && speed <= v_set
}
pub fn involuntary_override_condition(
    activation_req: ActivationResquest,
    kickdown: KickdownState,
    v_set: f64,
    speed: f64,
) -> bool {
    on_condition(activation_req) && kickdown != KickdownState::Activated && speed > v_set
}
pub fn voluntary_override_condition(
    activation_req: ActivationResquest,
    kickdown: KickdownState,
) -> bool {
    on_condition(activation_req) && kickdown == KickdownState::Activated
}
pub fn standby_condition(
    activation_req: ActivationResquest,
    vacuum_brake_state: VacuumBrakeState,
    v_set: f64,
) -> bool {
    activation_req == ActivationResquest::StandBy
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
    pub activation_req: ActivationResquest,
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
    pub activation_req: ActivationResquest,
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
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Context {
    pub state_update: bool,
    pub v_update: bool,
    pub activation: ActivationResquest,
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
    pub v_update: bool,
    pub set_speed: f64,
    pub on_state: SpeedLimiterOn,
    pub activation: ActivationResquest,
    pub v_set: f64,
    pub vacuum_brake: VacuumBrakeState,
    pub in_regulation_aux: bool,
    pub state_update: bool,
    pub v_set_aux: f64,
    pub kickdown: KickdownState,
    pub vdc: VdcState,
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
pub mod toto_service {
    use super::*;
    use futures::{sink::SinkExt, stream::StreamExt};
    use TotoServiceInput as I;
    use TotoServiceOutput as O;
    use TotoServiceTimer as T;
    #[derive(PartialEq)]
    pub enum TotoServiceTimer {
        period_fresh_ident,
    }
    impl TotoServiceTimer {
        pub fn get_duration(&self) -> std::time::Duration {
            match self {
                T::period_fresh_ident => std::time::Duration::from_millis(10u64),
            }
        }
    }
    impl priority_stream::Reset for TotoServiceTimer {
        fn do_reset(&self) -> bool {
            match self {
                T::period_fresh_ident => false,
            }
        }
    }
    pub enum TotoServiceInput {
        activation(ActivationResquest, std::time::Instant),
        set_speed(f64, std::time::Instant),
        speed(f64, std::time::Instant),
        vacuum_brake(VacuumBrakeState, std::time::Instant),
        kickdown(KickdownState, std::time::Instant),
        vdc(VdcState, std::time::Instant),
        timer(T, std::time::Instant),
    }
    impl priority_stream::Reset for TotoServiceInput {
        fn do_reset(&self) -> bool {
            match self {
                TotoServiceInput::timer(timer, _) => timer.do_reset(),
                _ => false,
            }
        }
    }
    impl PartialEq for TotoServiceInput {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (I::activation(this, _), I::activation(other, _)) => this.eq(other),
                (I::set_speed(this, _), I::set_speed(other, _)) => this.eq(other),
                (I::speed(this, _), I::speed(other, _)) => this.eq(other),
                (I::vacuum_brake(this, _), I::vacuum_brake(other, _)) => this.eq(other),
                (I::kickdown(this, _), I::kickdown(other, _)) => this.eq(other),
                (I::vdc(this, _), I::vdc(other, _)) => this.eq(other),
                (I::timer(this, _), I::timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl TotoServiceInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::activation(_, instant) => *instant,
                I::set_speed(_, instant) => *instant,
                I::speed(_, instant) => *instant,
                I::vacuum_brake(_, instant) => *instant,
                I::kickdown(_, instant) => *instant,
                I::vdc(_, instant) => *instant,
                I::timer(_, instant) => *instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    pub enum TotoServiceOutput {
        in_regulation(bool, std::time::Instant),
        v_set(f64, std::time::Instant),
    }
    pub struct TotoService {
        context: Context,
        process_set_speed: ProcessSetSpeedState,
        speed_limiter: SpeedLimiterState,
        output: futures::channel::mpsc::Sender<O>,
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl TotoService {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> TotoService {
            let context = Context::init();
            let process_set_speed = ProcessSetSpeedState::init();
            let speed_limiter = SpeedLimiterState::init();
            let period_fresh_ident =
                tokio::time::interval(tokio::time::Duration::from_millis(10u64));
            TotoService {
                context,
                process_set_speed,
                speed_limiter,
                output,
                timer,
            }
        }
        pub async fn run_loop(self, input: impl futures::Stream<Item = I>) {
            tokio::pin!(input);
            let mut service = self;
            loop {
                tokio::select! { input = input . next () => if let Some (input) = input { match input { I :: activation (activation , instant) => service . handle_activation (instant , activation) . await , I :: set_speed (set_speed , instant) => service . handle_set_speed (instant , set_speed) . await , I :: speed (speed , instant) => service . handle_speed (instant , speed) . await , I :: vacuum_brake (vacuum_brake , instant) => service . handle_vacuum_brake (instant , vacuum_brake) . await , I :: kickdown (kickdown , instant) => service . handle_kickdown (instant , kickdown) . await , I :: vdc (vdc , instant) => service . handle_vdc (instant , vdc) . await , I :: timer (T :: period_fresh_ident , instant) => service . handle_period_fresh_ident (instant) . await , } } else { break ; } }
            }
        }
        async fn handle_activation(
            &mut self,
            instant: std::time::Instant,
            activation: ActivationResquest,
        ) {
            self.context.activation = activation;
        }
        async fn handle_set_speed(&mut self, instant: std::time::Instant, set_speed: f64) {
            self.context.set_speed = set_speed;
        }
        async fn handle_speed(&mut self, instant: std::time::Instant, speed: f64) {
            self.context.speed = speed;
        }
        async fn handle_vacuum_brake(
            &mut self,
            instant: std::time::Instant,
            vacuum_brake: VacuumBrakeState,
        ) {
            self.context.vacuum_brake = vacuum_brake;
        }
        async fn handle_kickdown(&mut self, instant: std::time::Instant, kickdown: KickdownState) {
            self.context.kickdown = kickdown;
        }
        async fn handle_vdc(&mut self, instant: std::time::Instant, vdc: VdcState) {
            self.context.vdc = vdc;
        }
        async fn handle_period_fresh_ident(&mut self, instant: std::time::Instant) {
            let (state, on_state, in_regulation_aux, state_update) = self
                .speed_limiter
                .step(self.context.get_speed_limiter_inputs());
            self.context.state = state;
            self.context.on_state = on_state;
            self.context.in_regulation_aux = in_regulation_aux;
            self.context.state_update = state_update;
            let in_regulation = self.context.in_regulation_aux.clone();
            self.output
                .send(O::in_regulation(in_regulation, instant))
                .await
                .unwrap();
        }
    }
}
