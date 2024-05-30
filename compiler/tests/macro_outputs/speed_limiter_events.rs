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
pub enum Kickdown {
    #[default]
    Activated,
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Failure {
    #[default]
    Entering,
    Recovered,
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
pub fn activation_condition(vacuum_brake_state: VacuumBrakeState, v_set: f64) -> bool {
    vacuum_brake_state != VacuumBrakeState::BelowMinLevel && v_set > 0.0
}
pub fn standby_condition(vacuum_brake_state: VacuumBrakeState, v_set: f64) -> bool {
    vacuum_brake_state == VacuumBrakeState::BelowMinLevel || v_set <= 0.0
}
pub enum ProcessSetSpeedEvent {
    set_speed(f64),
    NoEvent,
}
pub struct ProcessSetSpeedInput {
    pub process_set_speed_event: ProcessSetSpeedEvent,
}
pub struct ProcessSetSpeedState {
    mem: f64,
}
impl ProcessSetSpeedState {
    pub fn init() -> ProcessSetSpeedState {
        ProcessSetSpeedState { mem: 0.0 }
    }
    pub fn step(&mut self, input: ProcessSetSpeedInput) -> (f64, bool) {
        let prev_v_set = self.mem;
        let (v_set, v_update) = match input.process_set_speed_event {
            ProcessSetSpeedEvent::set_speed(v) => {
                let v_set = (threshold_set_speed)(v);
                let v_update = prev_v_set != v_set;
                (v_set, v_update)
            }
            _ => {
                let v_set = prev_v_set;
                let v_update = false;
                (v_set, v_update)
            }
        };
        self.mem = v_set;
        (v_set, v_update)
    }
}
pub enum SpeedLimiterOnEvent {
    kickdown(Kickdown),
    NoEvent,
}
pub struct SpeedLimiterOnInput {
    pub prev_on_state: SpeedLimiterOn,
    pub vacuum_brake_state: VacuumBrakeState,
    pub speed: f64,
    pub v_set: f64,
    pub speed_limiter_on_event: SpeedLimiterOnEvent,
}
pub struct SpeedLimiterOnState {
    mem: Hysterisis,
}
impl SpeedLimiterOnState {
    pub fn init() -> SpeedLimiterOnState {
        SpeedLimiterOnState {
            mem: (new_hysterisis)(0.0),
        }
    }
    pub fn step(&mut self, input: SpeedLimiterOnInput) -> (SpeedLimiterOn, bool, bool) {
        let prev_hysterisis = self.mem;
        let (on_state, hysterisis, state_update) = match input.speed_limiter_on_event {
            SpeedLimiterOnEvent::kickdown(_) if input.prev_on_state == SpeedLimiterOn::Actif => {
                let state_update = true;
                let hysterisis = prev_hysterisis;
                let on_state = SpeedLimiterOn::OverrideVoluntary;
                (on_state, hysterisis, state_update)
            }
            _ => {
                let (on_state, hysterisis, state_update) = match input.prev_on_state {
                    SpeedLimiterOn::StandBy
                        if (activation_condition)(input.vacuum_brake_state, input.v_set) =>
                    {
                        let on_state = SpeedLimiterOn::Actif;
                        let hysterisis = (new_hysterisis)(0.0);
                        let state_update = true;
                        (on_state, hysterisis, state_update)
                    }
                    SpeedLimiterOn::OverrideVoluntary if input.speed <= input.v_set => {
                        let on_state = SpeedLimiterOn::Actif;
                        let hysterisis = (new_hysterisis)(0.0);
                        let state_update = true;
                        (on_state, hysterisis, state_update)
                    }
                    SpeedLimiterOn::Actif
                        if (standby_condition)(input.vacuum_brake_state, input.v_set) =>
                    {
                        let on_state = SpeedLimiterOn::StandBy;
                        let hysterisis = prev_hysterisis;
                        let state_update = true;
                        (on_state, hysterisis, state_update)
                    }
                    SpeedLimiterOn::Actif => {
                        let on_state = input.prev_on_state;
                        let hysterisis =
                            (update_hysterisis)(prev_hysterisis, input.speed, input.v_set);
                        let state_update = false;
                        (on_state, hysterisis, state_update)
                    }
                    _ => {
                        let on_state = input.prev_on_state;
                        let hysterisis = prev_hysterisis;
                        let state_update = false;
                        (on_state, hysterisis, state_update)
                    }
                };
                (on_state, hysterisis, state_update)
            }
        };
        let in_reg = (in_regulation)(hysterisis);
        self.mem = hysterisis;
        (on_state, in_reg, state_update)
    }
}
pub enum SpeedLimiterEvent {
    activation_req(ActivationResquest),
    kickdown(Kickdown),
    failure(Failure),
    NoEvent,
}
impl Into<SpeedLimiterOnEvent> for SpeedLimiterEvent {
    fn into(self) -> SpeedLimiterOnEvent {
        match self {
            SpeedLimiterEvent::kickdown(v) => SpeedLimiterOnEvent::kickdown(v),
            _ => SpeedLimiterOnEvent::NoEvent,
        }
    }
}
pub struct SpeedLimiterInput {
    pub vacuum_brake_state: VacuumBrakeState,
    pub vdc_disabled: VdcState,
    pub speed: f64,
    pub v_set: f64,
    pub speed_limiter_event: SpeedLimiterEvent,
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
        let prev_state = self.mem;
        let prev_on_state = self.mem_1;
        let prev_in_regulation = self.mem_2;
        let (state, on_state, in_regulation, state_update) = match input.speed_limiter_event {
            SpeedLimiterEvent::activation_req(ActivationResquest::Off) => {
                let state = SpeedLimiter::Off;
                let on_state = SpeedLimiterOn::StandBy;
                let in_regulation = false;
                let state_update = prev_state != SpeedLimiter::Off;
                (state, on_state, in_regulation, state_update)
            }
            SpeedLimiterEvent::activation_req(ActivationResquest::On)
                if prev_state == SpeedLimiter::Off =>
            {
                let state = SpeedLimiter::On;
                let on_state = SpeedLimiterOn::StandBy;
                let in_regulation = true;
                let state_update = true;
                (state, on_state, in_regulation, state_update)
            }
            SpeedLimiterEvent::failure(Failure::Entering) => {
                let state = SpeedLimiter::Fail;
                let on_state = SpeedLimiterOn::StandBy;
                let in_regulation = false;
                let state_update = prev_state != SpeedLimiter::Fail;
                (state, on_state, in_regulation, state_update)
            }
            SpeedLimiterEvent::failure(Failure::Recovered) if prev_state == SpeedLimiter::Fail => {
                let state = SpeedLimiter::On;
                let on_state = SpeedLimiterOn::StandBy;
                let in_regulation = true;
                let state_update = true;
                (state, on_state, in_regulation, state_update)
            }
            _ => {
                let (state, on_state, in_regulation, state_update) = match prev_state {
                    SpeedLimiter::On => {
                        let state = prev_state;
                        let (on_state, in_regulation, state_update) =
                            self.speed_limiter_on.step(SpeedLimiterOnInput {
                                prev_on_state: prev_on_state,
                                vacuum_brake_state: input.vacuum_brake_state,
                                speed: input.speed,
                                v_set: input.v_set,
                                speed_limiter_on_event: input.speed_limiter_event.into(),
                            });
                        (state, on_state, in_regulation, state_update)
                    }
                    _ => {
                        let state = prev_state;
                        let on_state = prev_on_state;
                        let in_regulation = prev_in_regulation;
                        let state_update = false;
                        (state, on_state, in_regulation, state_update)
                    }
                };
                (state, on_state, in_regulation, state_update)
            }
        };
        self.mem = state;
        self.mem_1 = on_state;
        self.mem_2 = in_regulation;
        (state, on_state, in_regulation, state_update)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Context {
    pub vacuum_brake: VacuumBrakeState,
    pub v_set_aux: f64,
    pub v_update: bool,
    pub in_regulation_aux: bool,
    pub state_update: bool,
    pub vdc: VdcState,
    pub state: SpeedLimiter,
    pub speed: f64,
    pub on_state: SpeedLimiterOn,
    pub v_set: f64,
}
impl Context {
    fn init() -> Context {
        Default::default()
    }
    fn get_speed_limiter_inputs(&self, event: SpeedLimiterEvent) -> SpeedLimiterInput {
        SpeedLimiterInput {
            vacuum_brake_state: self.vacuum_brake,
            vdc_disabled: self.vdc,
            speed: self.speed,
            v_set: self.v_set,
            speed_limiter_event: event,
        }
    }
    fn get_process_set_speed_inputs(&self, event: ProcessSetSpeedEvent) -> ProcessSetSpeedInput {
        ProcessSetSpeedInput {
            process_set_speed_event: event,
        }
    }
}
pub async fn run_toto_loop(
    mut activation_channel: tokio::sync::mpsc::Receiver<ActivationResquest>,
    mut set_speed_channel: tokio::sync::mpsc::Receiver<f64>,
    mut speed_channel: tokio::sync::mpsc::Receiver<f64>,
    mut vacuum_brake_channel: tokio::sync::mpsc::Receiver<VacuumBrakeState>,
    mut kickdown_channel: tokio::sync::mpsc::Receiver<Kickdown>,
    mut failure_channel: tokio::sync::mpsc::Receiver<Failure>,
    mut vdc_channel: tokio::sync::mpsc::Receiver<VdcState>,
    in_regulation_channel: tokio::sync::mpsc::Sender<bool>,
    v_set_channel: tokio::sync::mpsc::Sender<f64>,
) {
    let mut process_set_speed = ProcessSetSpeedState::init();
    let mut speed_limiter = SpeedLimiterState::init();
    let mut period = tokio::time::interval(std::time::Duration::from_millis(10u64));
    let mut context = Context::init();
    loop {
        tokio::select! { activation = activation_channel . recv () => { let activation = activation . unwrap () ; let (state , on_state , in_regulation_aux , state_update) = speed_limiter . step (context . get_speed_limiter_inputs (SpeedLimiterEvent :: activation_req (activation))) ; context . state = state ; context . on_state = on_state ; context . in_regulation_aux = in_regulation_aux ; context . state_update = state_update ; let state = context . state . clone () ; let on_state = context . on_state . clone () ; let in_regulation_aux = context . in_regulation_aux . clone () ; let state_update = context . state_update . clone () ; let in_regulation = in_regulation_aux ; in_regulation_channel . send (in_regulation) . await . unwrap () ; } set_speed = set_speed_channel . recv () => { let set_speed = set_speed . unwrap () ; let (v_set_aux , v_update) = process_set_speed . step (context . get_process_set_speed_inputs (ProcessSetSpeedEvent :: set_speed (set_speed))) ; context . v_set_aux = v_set_aux ; context . v_update = v_update ; let v_set_aux = context . v_set_aux . clone () ; let v_update = context . v_update . clone () ; let v_set = v_set_aux ; v_set_channel . send (v_set) . await . unwrap () ; let state = context . state . clone () ; let on_state = context . on_state . clone () ; let in_regulation_aux = context . in_regulation_aux . clone () ; let state_update = context . state_update . clone () ; let in_regulation = in_regulation_aux ; in_regulation_channel . send (in_regulation) . await . unwrap () ; } speed = speed_channel . recv () => { let speed = speed . unwrap () ; let state = context . state . clone () ; let on_state = context . on_state . clone () ; let in_regulation_aux = context . in_regulation_aux . clone () ; let state_update = context . state_update . clone () ; let in_regulation = in_regulation_aux ; in_regulation_channel . send (in_regulation) . await . unwrap () ; } vacuum_brake = vacuum_brake_channel . recv () => { let vacuum_brake = vacuum_brake . unwrap () ; let state = context . state . clone () ; let on_state = context . on_state . clone () ; let in_regulation_aux = context . in_regulation_aux . clone () ; let state_update = context . state_update . clone () ; let in_regulation = in_regulation_aux ; in_regulation_channel . send (in_regulation) . await . unwrap () ; } kickdown = kickdown_channel . recv () => { let kickdown = kickdown . unwrap () ; let (state , on_state , in_regulation_aux , state_update) = speed_limiter . step (context . get_speed_limiter_inputs (SpeedLimiterEvent :: kickdown (kickdown))) ; context . state = state ; context . on_state = on_state ; context . in_regulation_aux = in_regulation_aux ; context . state_update = state_update ; let state = context . state . clone () ; let on_state = context . on_state . clone () ; let in_regulation_aux = context . in_regulation_aux . clone () ; let state_update = context . state_update . clone () ; let in_regulation = in_regulation_aux ; in_regulation_channel . send (in_regulation) . await . unwrap () ; } failure = failure_channel . recv () => { let failure = failure . unwrap () ; let (state , on_state , in_regulation_aux , state_update) = speed_limiter . step (context . get_speed_limiter_inputs (SpeedLimiterEvent :: failure (failure))) ; context . state = state ; context . on_state = on_state ; context . in_regulation_aux = in_regulation_aux ; context . state_update = state_update ; let state = context . state . clone () ; let on_state = context . on_state . clone () ; let in_regulation_aux = context . in_regulation_aux . clone () ; let state_update = context . state_update . clone () ; let in_regulation = in_regulation_aux ; in_regulation_channel . send (in_regulation) . await . unwrap () ; } vdc = vdc_channel . recv () => { let vdc = vdc . unwrap () ; let state = context . state . clone () ; let on_state = context . on_state . clone () ; let in_regulation_aux = context . in_regulation_aux . clone () ; let state_update = context . state_update . clone () ; let in_regulation = in_regulation_aux ; in_regulation_channel . send (in_regulation) . await . unwrap () ; } _ = period . tick () => { let (state , on_state , in_regulation_aux , state_update) = speed_limiter . step (context . get_speed_limiter_inputs (SpeedLimiterEvent :: NoEvent)) ; context . state = state ; context . on_state = on_state ; context . in_regulation_aux = in_regulation_aux ; context . state_update = state_update ; let state = context . state . clone () ; let on_state = context . on_state . clone () ; let in_regulation_aux = context . in_regulation_aux . clone () ; let state_update = context . state_update . clone () ; let in_regulation = in_regulation_aux ; in_regulation_channel . send (in_regulation) . await . unwrap () ; } }
    }
}
