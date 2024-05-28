#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Hysterisis {
    pub value: f64,
    pub flag: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActivationResquest {
    Off,
    On,
    Initialization,
    StandBy,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VdcState {
    On,
    Off,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VacuumBrakeState {
    BelowMinLevel,
    AboveMinLevel,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KickdownState {
    Deactivated,
    Activated,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpeedLimiter {
    Off,
    On,
    Fail,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpeedLimiterOn {
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
    (on_condition)(activation_req) && kickdown != KickdownState::Activated && speed <= v_set
}
pub fn involuntary_override_condition(
    activation_req: ActivationResquest,
    kickdown: KickdownState,
    v_set: f64,
    speed: f64,
) -> bool {
    (on_condition)(activation_req) && kickdown != KickdownState::Activated && speed > v_set
}
pub fn voluntary_override_condition(
    activation_req: ActivationResquest,
    kickdown: KickdownState,
) -> bool {
    (on_condition)(activation_req) && kickdown == KickdownState::Activated
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
    mem_: f64,
}
impl ProcessSetSpeedState {
    pub fn init() -> ProcessSetSpeedState {
        ProcessSetSpeedState { mem_: 0.0 }
    }
    pub fn step(&mut self, input: ProcessSetSpeedInput) -> (f64, bool) {
        let v_set = (threshold_set_speed)(input.set_speed);
        let prev_v_set = self.mem_;
        let v_update = prev_v_set != v_set;
        self.mem_ = v_set;
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
    mem_: Hysterisis,
}
impl SpeedLimiterOnState {
    pub fn init() -> SpeedLimiterOnState {
        SpeedLimiterOnState {
            mem_: (new_hysterisis)(0.0),
        }
    }
    pub fn step(&mut self, input: SpeedLimiterOnInput) -> (SpeedLimiterOn, bool) {
        let prev_hysterisis = self.mem_;
        let (on_state, hysterisis) = match input.prev_on_state {
            SpeedLimiterOn::StandBy
                if (activation_condition)(
                    input.activation_req,
                    input.vacuum_brake_state,
                    input.v_set,
                ) =>
            {
                let hysterisis = (new_hysterisis)(0.0);
                let on_state = SpeedLimiterOn::Actif;
                (on_state, hysterisis)
            }
            SpeedLimiterOn::OverrideVoluntary
                if (exit_override_condition)(
                    input.activation_req,
                    input.kickdown,
                    input.v_set,
                    input.speed,
                ) =>
            {
                let hysterisis = (new_hysterisis)(0.0);
                let on_state = SpeedLimiterOn::Actif;
                (on_state, hysterisis)
            }
            SpeedLimiterOn::OverrideInvoluntary
                if (exit_override_condition)(
                    input.activation_req,
                    input.kickdown,
                    input.v_set,
                    input.speed,
                ) =>
            {
                let hysterisis = (new_hysterisis)(0.0);
                let on_state = SpeedLimiterOn::Actif;
                (on_state, hysterisis)
            }
            SpeedLimiterOn::OverrideVoluntary
                if (involuntary_override_condition)(
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
                if (voluntary_override_condition)(input.activation_req, input.kickdown) =>
            {
                let hysterisis = prev_hysterisis;
                let on_state = SpeedLimiterOn::OverrideVoluntary;
                (on_state, hysterisis)
            }
            SpeedLimiterOn::Actif
                if (standby_condition)(
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
                let hysterisis = (update_hysterisis)(prev_hysterisis, input.speed, input.v_set);
                let on_state = input.prev_on_state;
                (on_state, hysterisis)
            }
            _ => {
                let hysterisis = prev_hysterisis;
                let on_state = input.prev_on_state;
                (on_state, hysterisis)
            }
        };
        let in_reg = (in_regulation)(hysterisis);
        self.mem_ = hysterisis;
        (on_state, in_reg)
    }
}
pub struct SpeedLimiterInput {
    pub activation_req: ActivationResquest,
    pub vacuum_brake_state: VacuumBrakeState,
    pub kickdown: KickdownState,
    pub vdc_disabled: VdcState,
    pub failure: bool,
    pub speed: f64,
    pub v_set: f64,
}
pub struct SpeedLimiterState {
    mem_: SpeedLimiter,
    mem__1: SpeedLimiterOn,
    mem__2: bool,
    speed_limiter_on: SpeedLimiterOnState,
}
impl SpeedLimiterState {
    pub fn init() -> SpeedLimiterState {
        SpeedLimiterState {
            mem_: SpeedLimiter::Off,
            mem__1: SpeedLimiterOn::StandBy,
            mem__2: false,
            speed_limiter_on: SpeedLimiterOnState::init(),
        }
    }
    pub fn step(&mut self, input: SpeedLimiterInput) -> (SpeedLimiter, SpeedLimiterOn, bool, bool) {
        let prev_state = self.mem_;
        let prev_on_state = self.mem__1;
        let (state, on_state, in_regulation) = match prev_state {
            _ if (off_condition)(input.activation_req, input.vdc_disabled) => {
                let state = SpeedLimiter::Off;
                let on_state = prev_on_state;
                let in_regulation = false;
                (state, on_state, in_regulation)
            }
            SpeedLimiter::Off if (on_condition)(input.activation_req) => {
                let (state, on_state, in_regulation) = match input.failure {
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
            SpeedLimiter::On if input.failure => {
                let state = SpeedLimiter::Fail;
                let on_state = prev_on_state;
                let in_regulation = false;
                (state, on_state, in_regulation)
            }
            SpeedLimiter::Fail if !input.failure => {
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
                let in_regulation = self.mem__2;
                (state, on_state, in_regulation)
            }
        };
        let state_update = state != prev_state || on_state != prev_on_state;
        self.mem_ = state;
        self.mem__1 = on_state;
        self.mem__2 = in_regulation;
        (state, on_state, in_regulation, state_update)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Context {
    pub v_update: bool,
    pub v_set: f64,
    pub set_speed: f64,
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
}
pub async fn run_toto_loop(
    mut activation_channel: tokio::sync::mpsc::Receiver<ActivationResquest>,
    mut set_speed_channel: tokio::sync::mpsc::Receiver<f64>,
    mut speed_channel: tokio::sync::mpsc::Receiver<f64>,
    mut vacuum_brake_channel: tokio::sync::mpsc::Receiver<VacuumBrakeState>,
    mut kickdown_channel: tokio::sync::mpsc::Receiver<KickdownState>,
    mut vdc_channel: tokio::sync::mpsc::Receiver<VdcState>,
    mut v_set_channel: tokio::sync::mpsc::Sender<f64>,
    mut v_update_channel: tokio::sync::mpsc::Sender<bool>,
) {
    let process_set_speed = ProcessSetSpeedState::init();
    let mut context = Context::init();
    loop {
        tokio::select! {
            activation = activation_channel.recv() =>
            { let activation = activation.unwrap(); } set_speed =
            set_speed_channel.recv() =>
            {
                let set_speed = set_speed.unwrap(); let v_set =
                context.v_set.clone(); let v_update =
                context.v_update.clone();
                v_update_channel.send(v_update).await.unwrap(); let v_set =
                context.v_set.clone(); let v_update =
                context.v_update.clone();
                v_set_channel.send(v_set).await.unwrap();
            } speed = speed_channel.recv() => { let speed = speed.unwrap(); }
            vacuum_brake = vacuum_brake_channel.recv() =>
            { let vacuum_brake = vacuum_brake.unwrap(); } kickdown =
            kickdown_channel.recv() => { let kickdown = kickdown.unwrap(); }
            vdc = vdc_channel.recv() => { let vdc = vdc.unwrap(); }
        }
    }
}
