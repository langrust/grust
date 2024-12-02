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
            last_on_state: Default::default(),
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
