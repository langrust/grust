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
pub enum Kickdown {
    #[default]
    Activated,
    Deactivated,
}
#[derive(Clone, Copy, PartialEq, Default)]
pub enum Failure {
    #[default]
    Entering,
    Recovered,
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
    let flag = if prev_hyst.flag && (speed <= deactivation_threshold) {
        false
    } else {
        if !prev_hyst.flag && (speed >= activation_threshold) {
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
    (vacuum_brake_state != VacuumBrakeState::BelowMinLevel) && (v_set > 0.0)
}
pub fn standby_condition(vacuum_brake_state: VacuumBrakeState, v_set: f64) -> bool {
    (vacuum_brake_state == VacuumBrakeState::BelowMinLevel) || (v_set <= 0.0)
}
pub struct ProcessSetSpeedInput {
    pub set_speed: Option<f64>,
}
pub struct ProcessSetSpeedState {
    mem: f64,
    mem_1: f64,
}
impl ProcessSetSpeedState {
    pub fn init() -> ProcessSetSpeedState {
        ProcessSetSpeedState {
            mem: Default::default(),
            mem_1: Default::default(),
        }
    }
    pub fn step(&mut self, input: ProcessSetSpeedInput) -> (f64, bool) {
        let prev_v_set = self.mem;
        let v_set = match (input.set_speed) {
            (Some(v)) => {
                let v_set = threshold_set_speed(v);
                v_set
            }
            (_) => self.mem_1,
        };
        let v_update = prev_v_set != v_set;
        self.mem = v_set;
        self.mem_1 = v_set;
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
    mem: Hysterisis,
    mem_1: Kickdown,
}
impl SpeedLimiterOnState {
    pub fn init() -> SpeedLimiterOnState {
        SpeedLimiterOnState {
            mem: new_hysterisis(0.0),
            mem_1: Default::default(),
        }
    }
    pub fn step(&mut self, input: SpeedLimiterOnInput) -> (SpeedLimiterOn, bool, bool) {
        let prev_hysterisis = self.mem;
        let kickdown_state = match (input.kickdown) {
            (Some(Kickdown::Activated)) if input.prev_on_state == SpeedLimiterOn::Actif => {
                let kickdown_state = Kickdown::Activated;
                kickdown_state
            }
            (Some(Kickdown::Deactivated)) => {
                let kickdown_state = Kickdown::Deactivated;
                kickdown_state
            }
            (_) => self.mem_1,
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
                let hysterisis = new_hysterisis(0.0);
                (hysterisis, on_state)
            }
            SpeedLimiterOn::OverrideVoluntary if input.speed <= input.v_set => {
                let on_state = SpeedLimiterOn::Actif;
                let hysterisis = new_hysterisis(0.0);
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
        self.mem = hysterisis;
        self.mem_1 = kickdown_state;
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
    mem: SpeedLimiter,
    mem_1: SpeedLimiterOn,
    mem_2: SpeedLimiter,
    speed_limiter_on: SpeedLimiterOnState,
}
impl SpeedLimiterState {
    pub fn init() -> SpeedLimiterState {
        SpeedLimiterState {
            mem: Default::default(),
            mem_1: Default::default(),
            mem_2: Default::default(),
            speed_limiter_on: SpeedLimiterOnState::init(),
        }
    }
    pub fn step(&mut self, input: SpeedLimiterInput) -> (SpeedLimiter, SpeedLimiterOn, bool, bool) {
        let prev_state = self.mem;
        let prev_on_state = self.mem_1;
        let state = match (input.activation_req, input.failure) {
            (Some(ActivationRequest::Off), _) => {
                let state = SpeedLimiter::Off;
                state
            }
            (Some(ActivationRequest::On), _) if prev_state == SpeedLimiter::Off => {
                let state = SpeedLimiter::On;
                state
            }
            (_, Some(Failure::Entering)) => {
                let state = SpeedLimiter::Fail;
                state
            }
            (_, Some(Failure::Recovered)) if prev_state == SpeedLimiter::Fail => {
                let state = SpeedLimiter::On;
                state
            }
            (_, _) => self.mem_2,
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
        self.mem = state;
        self.mem_1 = on_state;
        self.mem_2 = state;
        (state, on_state, in_regulation, state_update)
    }
}
