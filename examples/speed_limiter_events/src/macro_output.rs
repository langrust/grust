#[derive(Clone, Copy, PartialEq, Default)]
pub struct Hysterisis {
    pub value: f64,
    pub flag: bool,
}
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ActivationResquest {
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
                let v_set = threshold_set_speed(v);
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
            mem: new_hysterisis(0.0),
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
                        if activation_condition(input.vacuum_brake_state, input.v_set) =>
                    {
                        let on_state = SpeedLimiterOn::Actif;
                        let hysterisis = new_hysterisis(0.0);
                        let state_update = true;
                        (on_state, hysterisis, state_update)
                    }
                    SpeedLimiterOn::OverrideVoluntary if input.speed <= input.v_set => {
                        let on_state = SpeedLimiterOn::Actif;
                        let hysterisis = new_hysterisis(0.0);
                        let state_update = true;
                        (on_state, hysterisis, state_update)
                    }
                    SpeedLimiterOn::Actif
                        if standby_condition(input.vacuum_brake_state, input.v_set) =>
                    {
                        let on_state = SpeedLimiterOn::StandBy;
                        let hysterisis = prev_hysterisis;
                        let state_update = true;
                        (on_state, hysterisis, state_update)
                    }
                    SpeedLimiterOn::Actif => {
                        let on_state = input.prev_on_state;
                        let hysterisis =
                            update_hysterisis(prev_hysterisis, input.speed, input.v_set);
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
        let in_reg = in_regulation(hysterisis);
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
#[derive(Clone, Copy, PartialEq, Default)]
pub struct Context {
    pub state_update: bool,
    pub v_update: bool,
    pub vacuum_brake: VacuumBrakeState,
    pub vdc: VdcState,
    pub speed: f64,
    pub changed_set_speed_old: f64,
    pub v_set_aux: f64,
    pub v_set: f64,
    pub flow_expression_fresh_ident: f64,
    pub in_regulation_aux: bool,
    pub on_state: SpeedLimiterOn,
    pub state: SpeedLimiter,
}
impl Context {
    fn init() -> Context {
        Default::default()
    }
    fn get_process_set_speed_inputs(&self, event: ProcessSetSpeedEvent) -> ProcessSetSpeedInput {
        ProcessSetSpeedInput {
            process_set_speed_event: event,
        }
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
    impl timer_stream::Timing for TotoServiceTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::period_fresh_ident => std::time::Duration::from_millis(10u64),
            }
        }
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
        kickdown(Kickdown, std::time::Instant),
        failure(Failure, std::time::Instant),
        vdc(VdcState, std::time::Instant),
        timer(T, std::time::Instant),
    }
    impl priority_stream::Reset for TotoServiceInput {
        fn do_reset(&self) -> bool {
            match self {
                TotoServiceInput::timer(timer, _) => timer_stream::Timing::do_reset(timer),
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
                (I::failure(this, _), I::failure(other, _)) => this.eq(other),
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
                I::failure(_, instant) => *instant,
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
                tokio::select! {
                    input = input.next() => if let Some(input) = input
                    {
                        match input
                        {
                            I :: activation(activation, instant) =>
                            service.handle_activation(instant, activation).await, I ::
                            set_speed(set_speed, instant) =>
                            service.handle_set_speed(instant, set_speed).await, I ::
                            speed(speed, instant) =>
                            service.handle_speed(instant, speed).await, I ::
                            vacuum_brake(vacuum_brake, instant) =>
                            service.handle_vacuum_brake(instant, vacuum_brake).await, I
                            :: kickdown(kickdown, instant) =>
                            service.handle_kickdown(instant, kickdown).await, I ::
                            failure(failure, instant) =>
                            service.handle_failure(instant, failure).await, I ::
                            vdc(vdc, instant) => service.handle_vdc(instant, vdc).await,
                            I :: timer(T :: period_fresh_ident, instant) =>
                            service.handle_period_fresh_ident(instant).await,
                        }
                    } else { break; }, _ = service.period_fresh_ident.tick() =>
                    service.handle_period_fresh_ident().await,
                }
            }
        }
        async fn handle_activation(
            &mut self,
            instant: std::time::Instant,
            activation: ActivationResquest,
        ) {
            let (state, on_state, in_regulation_aux, state_update) = self.speed_limiter.step(
                self.context
                    .get_speed_limiter_inputs(SpeedLimiterEvent::activation_req(activation)),
            );
            self.context.state = state;
            self.context.on_state = on_state;
            self.context.in_regulation_aux = in_regulation_aux;
            self.context.state_update = state_update;
            let in_regulation = self.context.in_regulation_aux.clone();
            {
                let res = self
                    .output
                    .send(O::in_regulation(in_regulation, instant))
                    .await;
                if res.is_err() {
                    return;
                }
            }
        }
        async fn handle_set_speed(&mut self, instant: std::time::Instant, set_speed: f64) {
            if (self.context.flow_expression_fresh_ident - set_speed) >= 1.0 {
                self.context.flow_expression_fresh_ident = set_speed;
            }
            let flow_expression_fresh_ident = self.context.flow_expression_fresh_ident.clone();
            if self.context.changed_set_speed_old != flow_expression_fresh_ident {
                let changed_set_speed = flow_expression_fresh_ident;
                self.context.changed_set_speed_old = flow_expression_fresh_ident;
                let (v_set_aux, v_update) =
                    self.process_set_speed
                        .step(self.context.get_process_set_speed_inputs(
                            ProcessSetSpeedEvent::set_speed(changed_set_speed),
                        ));
                self.context.v_set_aux = v_set_aux;
                self.context.v_update = v_update;
                let v_set = self.context.v_set_aux.clone();
                {
                    let res = self.output.send(O::v_set(v_set, instant)).await;
                    if res.is_err() {
                        return;
                    }
                }
            } else {
            }
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
        async fn handle_kickdown(&mut self, instant: std::time::Instant, kickdown: Kickdown) {
            let (state, on_state, in_regulation_aux, state_update) = self.speed_limiter.step(
                self.context
                    .get_speed_limiter_inputs(SpeedLimiterEvent::kickdown(kickdown)),
            );
            self.context.state = state;
            self.context.on_state = on_state;
            self.context.in_regulation_aux = in_regulation_aux;
            self.context.state_update = state_update;
            let in_regulation = self.context.in_regulation_aux.clone();
            {
                let res = self
                    .output
                    .send(O::in_regulation(in_regulation, instant))
                    .await;
                if res.is_err() {
                    return;
                }
            }
        }
        async fn handle_failure(&mut self, instant: std::time::Instant, failure: Failure) {
            let (state, on_state, in_regulation_aux, state_update) = self.speed_limiter.step(
                self.context
                    .get_speed_limiter_inputs(SpeedLimiterEvent::failure(failure)),
            );
            self.context.state = state;
            self.context.on_state = on_state;
            self.context.in_regulation_aux = in_regulation_aux;
            self.context.state_update = state_update;
            let in_regulation = self.context.in_regulation_aux.clone();
            {
                let res = self
                    .output
                    .send(O::in_regulation(in_regulation, instant))
                    .await;
                if res.is_err() {
                    return;
                }
            }
        }
        async fn handle_vdc(&mut self, instant: std::time::Instant, vdc: VdcState) {
            self.context.vdc = vdc;
        }
        async fn handle_period_fresh_ident(&mut self, instant: std::time::Instant) {
            let (state, on_state, in_regulation_aux, state_update) = self.speed_limiter.step(
                self.context
                    .get_speed_limiter_inputs(SpeedLimiterEvent::NoEvent),
            );
            self.context.state = state;
            self.context.on_state = on_state;
            self.context.in_regulation_aux = in_regulation_aux;
            self.context.state_update = state_update;
            let in_regulation = self.context.in_regulation_aux.clone();
            {
                let res = self
                    .output
                    .send(O::in_regulation(in_regulation, instant))
                    .await;
                if res.is_err() {
                    return;
                }
            }
        }
    }
}
