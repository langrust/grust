#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Activation {
    #[default]
    On,
    Off,
}
pub fn safety_distance(sv_v_m_s: f64, fv_v_m_s: f64) -> f64 {
    let sv_d_stop_m = (sv_v_m_s * 2.0f64) + ((sv_v_m_s * sv_v_m_s) / (2.0f64 * 5.886f64));
    let fv_d_stop_m = (fv_v_m_s * fv_v_m_s) / (2.0f64 * 5.886f64);
    sv_d_stop_m - fv_d_stop_m
}
pub struct DeriveInput {
    pub x: f64,
    pub t_ms: f64,
}
pub struct DeriveState {
    last_t_ms: f64,
    last_x: f64,
}
impl grust::core::Component for DeriveState {
    type Input = DeriveInput;
    type Output = f64;
    fn init() -> DeriveState {
        DeriveState {
            last_t_ms: 0.0f64,
            last_x: 0.0f64,
        }
    }
    fn step(&mut self, input: DeriveInput) -> f64 {
        let dt_ms = input.t_ms - self.last_t_ms;
        let v_ms = (input.x - self.last_x) / dt_ms;
        let v_s = v_ms / 1000.0f64;
        self.last_t_ms = input.t_ms;
        self.last_x = input.x;
        v_s
    }
}
pub struct IntegrateInput {
    pub v_s: f64,
    pub t_ms: f64,
}
pub struct IntegrateState {
    last_t_ms: f64,
    last_x: f64,
}
impl grust::core::Component for IntegrateState {
    type Input = IntegrateInput;
    type Output = f64;
    fn init() -> IntegrateState {
        IntegrateState {
            last_t_ms: 0.0f64,
            last_x: 0.0f64,
        }
    }
    fn step(&mut self, input: IntegrateInput) -> f64 {
        let v_ms = input.v_s * 1000.0f64;
        let dt_ms = input.t_ms - self.last_t_ms;
        let unbounded_x = self.last_x + (v_ms * dt_ms);
        let x = if unbounded_x > 10.0f64 {
            10.0f64
        } else {
            if unbounded_x < -10.0f64 {
                -10.0f64
            } else {
                unbounded_x
            }
        };
        self.last_t_ms = input.t_ms;
        self.last_x = x;
        x
    }
}
pub struct CommandInput {
    pub distance_m: f64,
    pub sv_v_km_h: f64,
    pub t_ms: f64,
}
pub struct CommandState {
    derive: DeriveState,
}
impl grust::core::Component for CommandState {
    type Input = CommandInput;
    type Output = f64;
    fn init() -> CommandState {
        CommandState {
            derive: <DeriveState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: CommandInput) -> f64 {
        let distancing_m_s = <DeriveState as grust::core::Component>::step(
            &mut self.derive,
            DeriveInput {
                x: input.distance_m,
                t_ms: input.t_ms,
            },
        );
        let sv_v_m_s = input.sv_v_km_h / 3.6f64;
        let fv_v_m_s = sv_v_m_s + distancing_m_s;
        let d_safe_m = safety_distance(sv_v_m_s, fv_v_m_s);
        let brakes_command = (distancing_m_s * distancing_m_s) / (input.distance_m - d_safe_m);
        brakes_command
    }
}
pub struct ErrorInput {
    pub sv_v_km_h: f64,
    pub brakes_m_s_command: f64,
    pub t_ms: f64,
}
pub struct ErrorState {
    derive: DeriveState,
}
impl grust::core::Component for ErrorState {
    type Input = ErrorInput;
    type Output = f64;
    fn init() -> ErrorState {
        ErrorState {
            derive: <DeriveState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: ErrorInput) -> f64 {
        let sv_v_m_s = input.sv_v_km_h / 3.6f64;
        let a_m_s = <DeriveState as grust::core::Component>::step(
            &mut self.derive,
            DeriveInput {
                x: sv_v_m_s,
                t_ms: input.t_ms,
            },
        );
        let a_m_s_command = -(input.brakes_m_s_command);
        let e_m_s = a_m_s_command - a_m_s;
        e_m_s
    }
}
pub struct PidInput {
    pub sv_v_km_h: f64,
    pub b_m_s_command: f64,
    pub t_ms: f64,
}
pub struct PidState {
    error: ErrorState,
    integrate: IntegrateState,
    derive: DeriveState,
}
impl grust::core::Component for PidState {
    type Input = PidInput;
    type Output = f64;
    fn init() -> PidState {
        PidState {
            error: <ErrorState as grust::core::Component>::init(),
            integrate: <IntegrateState as grust::core::Component>::init(),
            derive: <DeriveState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: PidInput) -> f64 {
        let p_e = <ErrorState as grust::core::Component>::step(
            &mut self.error,
            ErrorInput {
                sv_v_km_h: input.sv_v_km_h,
                brakes_m_s_command: input.b_m_s_command,
                t_ms: input.t_ms,
            },
        );
        let i_e = <IntegrateState as grust::core::Component>::step(
            &mut self.integrate,
            IntegrateInput {
                v_s: p_e,
                t_ms: input.t_ms,
            },
        );
        let d_e = <DeriveState as grust::core::Component>::step(
            &mut self.derive,
            DeriveInput {
                x: p_e,
                t_ms: input.t_ms,
            },
        );
        let b_m_s_control = ((1.0f64 * p_e) + (0.1f64 * i_e)) + (0.05f64 * d_e);
        b_m_s_control
    }
}
pub struct ActivateInput {
    pub acc_active: Option<Activation>,
    pub distance_m: f64,
}
pub struct ActivateState {
    last_active: bool,
    last_approaching: bool,
    last_distance_m: f64,
    last_x: bool,
    last_x_1: bool,
}
impl grust::core::Component for ActivateState {
    type Input = ActivateInput;
    type Output = bool;
    fn init() -> ActivateState {
        ActivateState {
            last_active: false,
            last_approaching: false,
            last_distance_m: 0.0f64,
            last_x: false,
            last_x_1: false,
        }
    }
    fn step(&mut self, input: ActivateInput) -> bool {
        let x = input.distance_m < self.last_distance_m;
        let x_1 = input.distance_m >= self.last_distance_m;
        let (active, approaching) = match (input.acc_active) {
            (Some(acc_active)) => {
                let active = acc_active == Activation::On;
                (active, self.last_approaching)
            }
            (_) if x && !(self.last_x) => {
                let approaching = true;
                (self.last_active, approaching)
            }
            (_) if x_1 && !(self.last_x_1) => {
                let approaching = false;
                (self.last_active, approaching)
            }
            (_) => (self.last_active, self.last_approaching),
        };
        let condition = active && approaching;
        self.last_active = active;
        self.last_approaching = approaching;
        self.last_distance_m = input.distance_m;
        self.last_x = x;
        self.last_x_1 = x_1;
        condition
    }
}
pub struct FilteredAccInput {
    pub condition: bool,
    pub distance_m: f64,
    pub sv_v_km_h: f64,
    pub t_ms: f64,
}
pub struct FilteredAccState {
    command: CommandState,
    pid: PidState,
}
impl grust::core::Component for FilteredAccState {
    type Input = FilteredAccInput;
    type Output = f64;
    fn init() -> FilteredAccState {
        FilteredAccState {
            command: <CommandState as grust::core::Component>::init(),
            pid: <PidState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: FilteredAccInput) -> f64 {
        let (brakes_command_m_s, brakes_m_s) = match input.condition {
            true => {
                let brakes_command_m_s = <CommandState as grust::core::Component>::step(
                    &mut self.command,
                    CommandInput {
                        distance_m: input.distance_m,
                        sv_v_km_h: input.sv_v_km_h,
                        t_ms: input.t_ms,
                    },
                );
                let brakes_m_s = <PidState as grust::core::Component>::step(
                    &mut self.pid,
                    PidInput {
                        sv_v_km_h: input.sv_v_km_h,
                        b_m_s_command: brakes_command_m_s,
                        t_ms: input.t_ms,
                    },
                );
                (brakes_command_m_s, brakes_m_s)
            }
            false => {
                let brakes_command_m_s = 0.0f64;
                let brakes_m_s = 0.0f64;
                (brakes_command_m_s, brakes_m_s)
            }
        };
        brakes_m_s
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
        DelayAdaptiveCruiseControl,
        TimeoutAdaptiveCruiseControl,
    }
    impl timer_stream::Timing for RuntimeTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::DelayAdaptiveCruiseControl => std::time::Duration::from_millis(10u64),
                T::TimeoutAdaptiveCruiseControl => std::time::Duration::from_millis(3000u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::DelayAdaptiveCruiseControl => true,
                T::TimeoutAdaptiveCruiseControl => true,
            }
        }
    }
    pub enum RuntimeInput {
        AccActive(Activation, std::time::Instant),
        SpeedKmH(f64, std::time::Instant),
        DistanceM(f64, std::time::Instant),
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
                (I::AccActive(this, _), I::AccActive(other, _)) => this.eq(other),
                (I::SpeedKmH(this, _), I::SpeedKmH(other, _)) => this.eq(other),
                (I::DistanceM(this, _), I::DistanceM(other, _)) => this.eq(other),
                (I::Timer(this, _), I::Timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::AccActive(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::SpeedKmH(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::DistanceM(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Timer(_, _grust_reserved_instant) => *_grust_reserved_instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    pub enum RuntimeOutput {
        BrakesMS(f64, std::time::Instant),
    }
    pub struct Runtime {
        adaptive_cruise_control: adaptive_cruise_control_service::AdaptiveCruiseControlService,
        output: futures::channel::mpsc::Sender<O>,
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl Runtime {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let adaptive_cruise_control =
                adaptive_cruise_control_service::AdaptiveCruiseControlService::init(
                    output.clone(),
                    timer.clone(),
                );
            Runtime {
                adaptive_cruise_control,
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
                .send_timer(
                    T::TimeoutAdaptiveCruiseControl,
                    _grust_reserved_init_instant,
                )
                .await?;
            runtime
                .send_output(O::BrakesMS(
                    Default::default(),
                    _grust_reserved_init_instant,
                ))
                .await?;
            while let Some(input) = input.next().await {
                match input {
                    I::Timer(T::DelayAdaptiveCruiseControl, _grust_reserved_instant) => {
                        runtime
                            .adaptive_cruise_control
                            .handle_delay_adaptive_cruise_control(_grust_reserved_instant)
                            .await?;
                    }
                    I::SpeedKmH(speed_km_h, _grust_reserved_instant) => {
                        runtime
                            .adaptive_cruise_control
                            .handle_speed_km_h(_grust_reserved_instant, speed_km_h)
                            .await?;
                    }
                    I::DistanceM(distance_m, _grust_reserved_instant) => {
                        runtime
                            .adaptive_cruise_control
                            .handle_distance_m(_grust_reserved_instant, distance_m)
                            .await?;
                    }
                    I::AccActive(acc_active, _grust_reserved_instant) => {
                        runtime
                            .adaptive_cruise_control
                            .handle_acc_active(_grust_reserved_instant, acc_active)
                            .await?;
                    }
                    I::Timer(T::TimeoutAdaptiveCruiseControl, _grust_reserved_instant) => {
                        runtime
                            .adaptive_cruise_control
                            .handle_timeout_adaptive_cruise_control(_grust_reserved_instant)
                            .await?;
                    }
                }
            }
            Ok(())
        }
    }
    pub mod adaptive_cruise_control_service {
        use super::*;
        use futures::{sink::SinkExt, stream::StreamExt};
        mod ctx_ty {
            use super::*;
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct Condition(bool, bool);
            impl Condition {
                pub fn set(&mut self, condition: bool) {
                    self.1 = self.0 != condition;
                    self.0 = condition;
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
            pub struct SpeedKmH(f64, bool);
            impl SpeedKmH {
                pub fn set(&mut self, speed_km_h: f64) {
                    self.1 = self.0 != speed_km_h;
                    self.0 = speed_km_h;
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
            pub struct T(f64, bool);
            impl T {
                pub fn set(&mut self, t: f64) {
                    self.1 = self.0 != t;
                    self.0 = t;
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
            pub struct BrakesMS(f64, bool);
            impl BrakesMS {
                pub fn set(&mut self, brakes_m_s: f64) {
                    self.1 = self.0 != brakes_m_s;
                    self.0 = brakes_m_s;
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
            pub struct DistanceM(f64, bool);
            impl DistanceM {
                pub fn set(&mut self, distance_m: f64) {
                    self.1 = self.0 != distance_m;
                    self.0 = distance_m;
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
        }
        #[derive(Clone, Copy, PartialEq, Default, Debug)]
        pub struct Context {
            pub condition: ctx_ty::Condition,
            pub speed_km_h: ctx_ty::SpeedKmH,
            pub t: ctx_ty::T,
            pub brakes_m_s: ctx_ty::BrakesMS,
            pub distance_m: ctx_ty::DistanceM,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.condition.reset();
                self.speed_km_h.reset();
                self.t.reset();
                self.brakes_m_s.reset();
                self.distance_m.reset();
            }
        }
        #[derive(Default)]
        pub struct AdaptiveCruiseControlServiceStore {
            acc_active: Option<(Activation, std::time::Instant)>,
            speed_km_h: Option<(f64, std::time::Instant)>,
            distance_m: Option<(f64, std::time::Instant)>,
        }
        impl AdaptiveCruiseControlServiceStore {
            pub fn not_empty(&self) -> bool {
                self.acc_active.is_some() || self.speed_km_h.is_some() || self.distance_m.is_some()
            }
        }
        pub struct AdaptiveCruiseControlService {
            begin: std::time::Instant,
            context: Context,
            delayed: bool,
            input_store: AdaptiveCruiseControlServiceStore,
            activate: ActivateState,
            filtered_acc: FilteredAccState,
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        }
        impl AdaptiveCruiseControlService {
            pub fn init(
                output: futures::channel::mpsc::Sender<O>,
                timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
            ) -> AdaptiveCruiseControlService {
                let context = Context::init();
                let delayed = true;
                let input_store = Default::default();
                let activate = <ActivateState as grust::core::Component>::init();
                let filtered_acc = <FilteredAccState as grust::core::Component>::init();
                AdaptiveCruiseControlService {
                    begin: std::time::Instant::now(),
                    context,
                    delayed,
                    input_store,
                    activate,
                    filtered_acc,
                    output,
                    timer,
                }
            }
            pub async fn handle_acc_active(
                &mut self,
                _acc_active_instant: std::time::Instant,
                acc_active: Activation,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_acc_active_instant).await?;
                    self.context.reset();
                    let acc_active_ref = &mut None;
                    *acc_active_ref = Some(acc_active);
                    if acc_active_ref.is_some() || self.context.distance_m.is_new() {
                        let condition = <ActivateState as grust::core::Component>::step(
                            &mut self.activate,
                            ActivateInput {
                                acc_active: *acc_active_ref,
                                distance_m: self.context.distance_m.get(),
                            },
                        );
                        self.context.condition.set(condition);
                    }
                    let t = (_acc_active_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.t.set(t);
                    if self.context.condition.is_new()
                        || self.context.distance_m.is_new()
                        || self.context.speed_km_h.is_new()
                        || self.context.t.is_new()
                    {
                        let brakes_m_s = <FilteredAccState as grust::core::Component>::step(
                            &mut self.filtered_acc,
                            FilteredAccInput {
                                condition: self.context.condition.get(),
                                distance_m: self.context.distance_m.get(),
                                sv_v_km_h: self.context.speed_km_h.get(),
                                t_ms: t,
                            },
                        );
                        self.context.brakes_m_s.set(brakes_m_s);
                    }
                    if self.context.brakes_m_s.is_new() {
                        self.send_output(
                            O::BrakesMS(self.context.brakes_m_s.get(), _acc_active_instant),
                            _acc_active_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .acc_active
                        .replace((acc_active, _acc_active_instant));
                    assert!(unique.is_none(), "flow `acc_active` changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_delay_adaptive_cruise_control(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.context.reset();
                if self.input_store.not_empty() {
                    self.reset_time_constraints(_grust_reserved_instant).await?;
                    match (
                        self.input_store.acc_active.take(),
                        self.input_store.speed_km_h.take(),
                        self.input_store.distance_m.take(),
                    ) {
                        (None, None, None) => {}
                        (Some((acc_active, _acc_active_instant)), None, None) => {
                            let acc_active_ref = &mut None;
                            *acc_active_ref = Some(acc_active);
                            if acc_active_ref.is_some() || self.context.distance_m.is_new() {
                                let condition = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        acc_active: *acc_active_ref,
                                        distance_m: self.context.distance_m.get(),
                                    },
                                );
                                self.context.condition.set(condition);
                            }
                            let t =
                                (_acc_active_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.t.set(t);
                            if self.context.condition.is_new()
                                || self.context.distance_m.is_new()
                                || self.context.speed_km_h.is_new()
                                || self.context.t.is_new()
                            {
                                let brakes_m_s = <FilteredAccState as grust::core::Component>::step(
                                    &mut self.filtered_acc,
                                    FilteredAccInput {
                                        condition: self.context.condition.get(),
                                        distance_m: self.context.distance_m.get(),
                                        sv_v_km_h: self.context.speed_km_h.get(),
                                        t_ms: t,
                                    },
                                );
                                self.context.brakes_m_s.set(brakes_m_s);
                            }
                            if self.context.brakes_m_s.is_new() {
                                self.send_output(
                                    O::BrakesMS(
                                        self.context.brakes_m_s.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, Some((speed_km_h, _speed_km_h_instant)), None) => {
                            self.context.speed_km_h.set(speed_km_h);
                            let t =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.t.set(t);
                            if self.context.condition.is_new()
                                || self.context.distance_m.is_new()
                                || self.context.speed_km_h.is_new()
                                || self.context.t.is_new()
                            {
                                let brakes_m_s = <FilteredAccState as grust::core::Component>::step(
                                    &mut self.filtered_acc,
                                    FilteredAccInput {
                                        condition: self.context.condition.get(),
                                        distance_m: self.context.distance_m.get(),
                                        sv_v_km_h: speed_km_h,
                                        t_ms: t,
                                    },
                                );
                                self.context.brakes_m_s.set(brakes_m_s);
                            }
                            if self.context.brakes_m_s.is_new() {
                                self.send_output(
                                    O::BrakesMS(
                                        self.context.brakes_m_s.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((acc_active, _acc_active_instant)),
                            Some((speed_km_h, _speed_km_h_instant)),
                            None,
                        ) => {
                            let acc_active_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            *acc_active_ref = Some(acc_active);
                            if acc_active_ref.is_some() || self.context.distance_m.is_new() {
                                let condition = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        acc_active: *acc_active_ref,
                                        distance_m: self.context.distance_m.get(),
                                    },
                                );
                                self.context.condition.set(condition);
                            }
                            let t =
                                (_acc_active_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.t.set(t);
                            if self.context.condition.is_new()
                                || self.context.distance_m.is_new()
                                || self.context.speed_km_h.is_new()
                                || self.context.t.is_new()
                            {
                                let brakes_m_s = <FilteredAccState as grust::core::Component>::step(
                                    &mut self.filtered_acc,
                                    FilteredAccInput {
                                        condition: self.context.condition.get(),
                                        distance_m: self.context.distance_m.get(),
                                        sv_v_km_h: speed_km_h,
                                        t_ms: t,
                                    },
                                );
                                self.context.brakes_m_s.set(brakes_m_s);
                            }
                            if self.context.brakes_m_s.is_new() {
                                self.send_output(
                                    O::BrakesMS(
                                        self.context.brakes_m_s.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, None, Some((distance_m, _distance_m_instant))) => {
                            self.context.distance_m.set(distance_m);
                            if self.context.distance_m.is_new() {
                                let condition = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        acc_active: None,
                                        distance_m: distance_m,
                                    },
                                );
                                self.context.condition.set(condition);
                            }
                            let t =
                                (_distance_m_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.t.set(t);
                            if self.context.condition.is_new()
                                || self.context.distance_m.is_new()
                                || self.context.speed_km_h.is_new()
                                || self.context.t.is_new()
                            {
                                let brakes_m_s = <FilteredAccState as grust::core::Component>::step(
                                    &mut self.filtered_acc,
                                    FilteredAccInput {
                                        condition: self.context.condition.get(),
                                        distance_m: distance_m,
                                        sv_v_km_h: self.context.speed_km_h.get(),
                                        t_ms: t,
                                    },
                                );
                                self.context.brakes_m_s.set(brakes_m_s);
                            }
                            if self.context.brakes_m_s.is_new() {
                                self.send_output(
                                    O::BrakesMS(
                                        self.context.brakes_m_s.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((acc_active, _acc_active_instant)),
                            None,
                            Some((distance_m, _distance_m_instant)),
                        ) => {
                            let acc_active_ref = &mut None;
                            self.context.distance_m.set(distance_m);
                            *acc_active_ref = Some(acc_active);
                            if acc_active_ref.is_some() || self.context.distance_m.is_new() {
                                let condition = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        acc_active: *acc_active_ref,
                                        distance_m: distance_m,
                                    },
                                );
                                self.context.condition.set(condition);
                            }
                            let t =
                                (_acc_active_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.t.set(t);
                            if self.context.condition.is_new()
                                || self.context.distance_m.is_new()
                                || self.context.speed_km_h.is_new()
                                || self.context.t.is_new()
                            {
                                let brakes_m_s = <FilteredAccState as grust::core::Component>::step(
                                    &mut self.filtered_acc,
                                    FilteredAccInput {
                                        condition: self.context.condition.get(),
                                        distance_m: distance_m,
                                        sv_v_km_h: self.context.speed_km_h.get(),
                                        t_ms: t,
                                    },
                                );
                                self.context.brakes_m_s.set(brakes_m_s);
                            }
                            if self.context.brakes_m_s.is_new() {
                                self.send_output(
                                    O::BrakesMS(
                                        self.context.brakes_m_s.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((speed_km_h, _speed_km_h_instant)),
                            Some((distance_m, _distance_m_instant)),
                        ) => {
                            self.context.distance_m.set(distance_m);
                            if self.context.distance_m.is_new() {
                                let condition = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        acc_active: None,
                                        distance_m: distance_m,
                                    },
                                );
                                self.context.condition.set(condition);
                            }
                            self.context.speed_km_h.set(speed_km_h);
                            let t =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.t.set(t);
                            if self.context.condition.is_new()
                                || self.context.distance_m.is_new()
                                || self.context.speed_km_h.is_new()
                                || self.context.t.is_new()
                            {
                                let brakes_m_s = <FilteredAccState as grust::core::Component>::step(
                                    &mut self.filtered_acc,
                                    FilteredAccInput {
                                        condition: self.context.condition.get(),
                                        distance_m: distance_m,
                                        sv_v_km_h: speed_km_h,
                                        t_ms: t,
                                    },
                                );
                                self.context.brakes_m_s.set(brakes_m_s);
                            }
                            if self.context.brakes_m_s.is_new() {
                                self.send_output(
                                    O::BrakesMS(
                                        self.context.brakes_m_s.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((acc_active, _acc_active_instant)),
                            Some((speed_km_h, _speed_km_h_instant)),
                            Some((distance_m, _distance_m_instant)),
                        ) => {
                            let acc_active_ref = &mut None;
                            self.context.distance_m.set(distance_m);
                            self.context.speed_km_h.set(speed_km_h);
                            *acc_active_ref = Some(acc_active);
                            if acc_active_ref.is_some() || self.context.distance_m.is_new() {
                                let condition = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        acc_active: *acc_active_ref,
                                        distance_m: distance_m,
                                    },
                                );
                                self.context.condition.set(condition);
                            }
                            let t =
                                (_acc_active_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.t.set(t);
                            if self.context.condition.is_new()
                                || self.context.distance_m.is_new()
                                || self.context.speed_km_h.is_new()
                                || self.context.t.is_new()
                            {
                                let brakes_m_s = <FilteredAccState as grust::core::Component>::step(
                                    &mut self.filtered_acc,
                                    FilteredAccInput {
                                        condition: self.context.condition.get(),
                                        distance_m: distance_m,
                                        sv_v_km_h: speed_km_h,
                                        t_ms: t,
                                    },
                                );
                                self.context.brakes_m_s.set(brakes_m_s);
                            }
                            if self.context.brakes_m_s.is_new() {
                                self.send_output(
                                    O::BrakesMS(
                                        self.context.brakes_m_s.get(),
                                        _grust_reserved_instant,
                                    ),
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
                    .send((T::DelayAdaptiveCruiseControl, _grust_reserved_instant))
                    .await?;
                Ok(())
            }
            pub async fn handle_speed_km_h(
                &mut self,
                _speed_km_h_instant: std::time::Instant,
                speed_km_h: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_speed_km_h_instant).await?;
                    self.context.reset();
                    self.context.speed_km_h.set(speed_km_h);
                    let t = (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.t.set(t);
                    if self.context.condition.is_new()
                        || self.context.distance_m.is_new()
                        || self.context.speed_km_h.is_new()
                        || self.context.t.is_new()
                    {
                        let brakes_m_s = <FilteredAccState as grust::core::Component>::step(
                            &mut self.filtered_acc,
                            FilteredAccInput {
                                condition: self.context.condition.get(),
                                distance_m: self.context.distance_m.get(),
                                sv_v_km_h: speed_km_h,
                                t_ms: t,
                            },
                        );
                        self.context.brakes_m_s.set(brakes_m_s);
                    }
                    if self.context.brakes_m_s.is_new() {
                        self.send_output(
                            O::BrakesMS(self.context.brakes_m_s.get(), _speed_km_h_instant),
                            _speed_km_h_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .speed_km_h
                        .replace((speed_km_h, _speed_km_h_instant));
                    assert!(unique.is_none(), "flow `speed_km_h` changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_timeout_adaptive_cruise_control(
                &mut self,
                _timeout_adaptive_cruise_control_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constraints(_timeout_adaptive_cruise_control_instant)
                    .await?;
                self.context.reset();
                if self.context.distance_m.is_new() {
                    let condition = <ActivateState as grust::core::Component>::step(
                        &mut self.activate,
                        ActivateInput {
                            acc_active: None,
                            distance_m: self.context.distance_m.get(),
                        },
                    );
                    self.context.condition.set(condition);
                }
                let t = (_timeout_adaptive_cruise_control_instant
                    .duration_since(self.begin)
                    .as_millis()) as f64;
                self.context.t.set(t);
                if self.context.condition.is_new()
                    || self.context.distance_m.is_new()
                    || self.context.speed_km_h.is_new()
                    || self.context.t.is_new()
                {
                    let brakes_m_s = <FilteredAccState as grust::core::Component>::step(
                        &mut self.filtered_acc,
                        FilteredAccInput {
                            condition: self.context.condition.get(),
                            distance_m: self.context.distance_m.get(),
                            sv_v_km_h: self.context.speed_km_h.get(),
                            t_ms: t,
                        },
                    );
                    self.context.brakes_m_s.set(brakes_m_s);
                }
                self.send_output(
                    O::BrakesMS(
                        self.context.brakes_m_s.get(),
                        _timeout_adaptive_cruise_control_instant,
                    ),
                    _timeout_adaptive_cruise_control_instant,
                )
                .await?;
                Ok(())
            }
            #[inline]
            pub async fn reset_service_timeout(
                &mut self,
                _timeout_adaptive_cruise_control_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer
                    .send((
                        T::TimeoutAdaptiveCruiseControl,
                        _timeout_adaptive_cruise_control_instant,
                    ))
                    .await?;
                Ok(())
            }
            pub async fn handle_distance_m(
                &mut self,
                _distance_m_instant: std::time::Instant,
                distance_m: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_distance_m_instant).await?;
                    self.context.reset();
                    self.context.distance_m.set(distance_m);
                    if self.context.distance_m.is_new() {
                        let condition = <ActivateState as grust::core::Component>::step(
                            &mut self.activate,
                            ActivateInput {
                                acc_active: None,
                                distance_m: distance_m,
                            },
                        );
                        self.context.condition.set(condition);
                    }
                    let t = (_distance_m_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.t.set(t);
                    if self.context.condition.is_new()
                        || self.context.distance_m.is_new()
                        || self.context.speed_km_h.is_new()
                        || self.context.t.is_new()
                    {
                        let brakes_m_s = <FilteredAccState as grust::core::Component>::step(
                            &mut self.filtered_acc,
                            FilteredAccInput {
                                condition: self.context.condition.get(),
                                distance_m: distance_m,
                                sv_v_km_h: self.context.speed_km_h.get(),
                                t_ms: t,
                            },
                        );
                        self.context.brakes_m_s.set(brakes_m_s);
                    }
                    if self.context.brakes_m_s.is_new() {
                        self.send_output(
                            O::BrakesMS(self.context.brakes_m_s.get(), _distance_m_instant),
                            _distance_m_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .distance_m
                        .replace((distance_m, _distance_m_instant));
                    assert!(unique.is_none(), "flow `distance_m` changes too frequently");
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
