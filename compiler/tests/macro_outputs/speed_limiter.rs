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
pub struct TotoService {
    context: Context,
    process_set_speed: ProcessSetSpeedState,
    speed_limiter: SpeedLimiterState,
    period: tokio::time::Interval,
    activation_channel: tokio::sync::mpsc::Receiver<ActivationResquest>,
    set_speed_channel: tokio::sync::mpsc::Receiver<f64>,
    speed_channel: tokio::sync::mpsc::Receiver<f64>,
    vacuum_brake_channel: tokio::sync::mpsc::Receiver<VacuumBrakeState>,
    kickdown_channel: tokio::sync::mpsc::Receiver<KickdownState>,
    vdc_channel: tokio::sync::mpsc::Receiver<VdcState>,
    in_regulation_channel: tokio::sync::mpsc::Sender<bool>,
    v_set_channel: tokio::sync::mpsc::Sender<f64>,
}
impl TotoService {
    fn new(
        activation_channel: tokio::sync::mpsc::Receiver<ActivationResquest>,
        set_speed_channel: tokio::sync::mpsc::Receiver<f64>,
        speed_channel: tokio::sync::mpsc::Receiver<f64>,
        vacuum_brake_channel: tokio::sync::mpsc::Receiver<VacuumBrakeState>,
        kickdown_channel: tokio::sync::mpsc::Receiver<KickdownState>,
        vdc_channel: tokio::sync::mpsc::Receiver<VdcState>,
        in_regulation_channel: tokio::sync::mpsc::Sender<bool>,
        v_set_channel: tokio::sync::mpsc::Sender<f64>,
    ) -> TotoService {
        let process_set_speed = ProcessSetSpeedState::init();
        let speed_limiter = SpeedLimiterState::init();
        let period = tokio::time::interval(tokio::time::Duration::from_millis(10u64));
        let context = Context::init();
        TotoService {
            context,
            process_set_speed,
            speed_limiter,
            period,
            activation_channel,
            set_speed_channel,
            speed_channel,
            vacuum_brake_channel,
            kickdown_channel,
            vdc_channel,
            in_regulation_channel,
            v_set_channel,
        }
    }
    pub async fn run_loop(
        activation_channel: tokio::sync::mpsc::Receiver<ActivationResquest>,
        set_speed_channel: tokio::sync::mpsc::Receiver<f64>,
        speed_channel: tokio::sync::mpsc::Receiver<f64>,
        vacuum_brake_channel: tokio::sync::mpsc::Receiver<VacuumBrakeState>,
        kickdown_channel: tokio::sync::mpsc::Receiver<KickdownState>,
        vdc_channel: tokio::sync::mpsc::Receiver<VdcState>,
        in_regulation_channel: tokio::sync::mpsc::Sender<bool>,
        v_set_channel: tokio::sync::mpsc::Sender<f64>,
    ) {
        let mut service = TotoService::new(
            activation_channel,
            set_speed_channel,
            speed_channel,
            vacuum_brake_channel,
            kickdown_channel,
            vdc_channel,
            in_regulation_channel,
            v_set_channel,
        );
        loop {
            tokio::select! { activation = service . activation_channel . recv () => service . handle_activation (activation . unwrap ()) . await , set_speed = service . set_speed_channel . recv () => service . handle_set_speed (set_speed . unwrap ()) . await , speed = service . speed_channel . recv () => service . handle_speed (speed . unwrap ()) . await , vacuum_brake = service . vacuum_brake_channel . recv () => service . handle_vacuum_brake (vacuum_brake . unwrap ()) . await , kickdown = service . kickdown_channel . recv () => service . handle_kickdown (kickdown . unwrap ()) . await , vdc = service . vdc_channel . recv () => service . handle_vdc (vdc . unwrap ()) . await , _ = service . period . tick () => service . handle_period () . await , }
        }
    }
    async fn handle_activation(&mut self, activation: ActivationResquest) {
        self.context.activation = activation;
    }
    async fn handle_set_speed(&mut self, set_speed: f64) {
        self.context.set_speed = set_speed;
    }
    async fn handle_speed(&mut self, speed: f64) {
        self.context.speed = speed;
    }
    async fn handle_vacuum_brake(&mut self, vacuum_brake: VacuumBrakeState) {
        self.context.vacuum_brake = vacuum_brake;
    }
    async fn handle_kickdown(&mut self, kickdown: KickdownState) {
        self.context.kickdown = kickdown;
    }
    async fn handle_vdc(&mut self, vdc: VdcState) {
        self.context.vdc = vdc;
    }
    async fn handle_period(&mut self) {
        let (state, on_state, in_regulation_aux, state_update) = self
            .speed_limiter
            .step(self.context.get_speed_limiter_inputs());
        self.context.state = state;
        self.context.on_state = on_state;
        self.context.in_regulation_aux = in_regulation_aux;
        self.context.state_update = state_update;
        let in_regulation = self.context.in_regulation_aux.clone();
        self.in_regulation_channel
            .send(in_regulation)
            .await
            .unwrap();
    }
}
