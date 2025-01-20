pub fn safety_distance(sv_v_m_s: f64, fv_v_m_s: f64) -> f64 {
    let rho_s = 2.0f64;
    let g = 9.81f64;
    let brake_max = 0.6f64 * g;
    let sv_d_stop_m = (sv_v_m_s * rho_s) + ((sv_v_m_s * sv_v_m_s) / (2.0f64 * brake_max));
    let fv_d_stop_m = (fv_v_m_s * fv_v_m_s) / (2.0f64 * brake_max);
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
impl DeriveState {
    pub fn init() -> DeriveState {
        DeriveState {
            last_t_ms: 0.0f64,
            last_x: 0.0f64,
        }
    }
    pub fn step(&mut self, input: DeriveInput) -> f64 {
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
impl IntegrateState {
    pub fn init() -> IntegrateState {
        IntegrateState {
            last_t_ms: 0.0f64,
            last_x: 0.0f64,
        }
    }
    pub fn step(&mut self, input: IntegrateInput) -> f64 {
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
impl CommandState {
    pub fn init() -> CommandState {
        CommandState {
            derive: DeriveState::init(),
        }
    }
    pub fn step(&mut self, input: CommandInput) -> f64 {
        let distancing_m_s = self.derive.step(DeriveInput {
            x: input.distance_m,
            t_ms: input.t_ms,
        });
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
impl ErrorState {
    pub fn init() -> ErrorState {
        ErrorState {
            derive: DeriveState::init(),
        }
    }
    pub fn step(&mut self, input: ErrorInput) -> f64 {
        let sv_v_m_s = input.sv_v_km_h / 3.6f64;
        let a_m_s = self.derive.step(DeriveInput {
            x: sv_v_m_s,
            t_ms: input.t_ms,
        });
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
impl PidState {
    pub fn init() -> PidState {
        PidState {
            error: ErrorState::init(),
            integrate: IntegrateState::init(),
            derive: DeriveState::init(),
        }
    }
    pub fn step(&mut self, input: PidInput) -> f64 {
        let p_e = self.error.step(ErrorInput {
            sv_v_km_h: input.sv_v_km_h,
            brakes_m_s_command: input.b_m_s_command,
            t_ms: input.t_ms,
        });
        let i_e = self.integrate.step(IntegrateInput {
            v_s: p_e,
            t_ms: input.t_ms,
        });
        let d_e = self.derive.step(DeriveInput {
            x: p_e,
            t_ms: input.t_ms,
        });
        let b_m_s_control = ((1.0f64 * p_e) + (0.1f64 * i_e)) + (0.05f64 * d_e);
        b_m_s_control
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
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl Runtime {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let adaptive_cruise_control =
                adaptive_cruise_control_service::AdaptiveCruiseControlService::init(
                    output,
                    timer.clone(),
                );
            Runtime {
                adaptive_cruise_control,
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
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct SpeedKmH(f64, bool);
        impl SpeedKmH {
            fn set(&mut self, speed_km_h: f64) {
                self.0 = speed_km_h;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct X1(f64, bool);
        impl X1 {
            fn set(&mut self, x_1: f64) {
                self.0 = x_1;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct BrakesMS(f64, bool);
        impl BrakesMS {
            fn set(&mut self, brakes_m_s: f64) {
                self.0 = brakes_m_s;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct BrakesCommandMS(f64, bool);
        impl BrakesCommandMS {
            fn set(&mut self, brakes_command_m_s: f64) {
                self.0 = brakes_command_m_s;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct DistanceM(f64, bool);
        impl DistanceM {
            fn set(&mut self, distance_m: f64) {
                self.0 = distance_m;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct X(f64, bool);
        impl X {
            fn set(&mut self, x: f64) {
                self.0 = x;
                self.1 = true;
            }
            fn get(&self) -> f64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Context {
            pub speed_km_h: SpeedKmH,
            pub x_1: X1,
            pub brakes_m_s: BrakesMS,
            pub brakes_command_m_s: BrakesCommandMS,
            pub distance_m: DistanceM,
            pub x: X,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.speed_km_h.reset();
                self.x_1.reset();
                self.brakes_m_s.reset();
                self.brakes_command_m_s.reset();
                self.distance_m.reset();
                self.x.reset();
            }
        }
        #[derive(Default)]
        pub struct AdaptiveCruiseControlServiceStore {
            speed_km_h: Option<(f64, std::time::Instant)>,
            distance_m: Option<(f64, std::time::Instant)>,
        }
        impl AdaptiveCruiseControlServiceStore {
            pub fn not_empty(&self) -> bool {
                self.speed_km_h.is_some() || self.distance_m.is_some()
            }
        }
        pub struct AdaptiveCruiseControlService {
            begin: std::time::Instant,
            context: Context,
            delayed: bool,
            input_store: AdaptiveCruiseControlServiceStore,
            pid: PidState,
            command: CommandState,
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
                let pid = PidState::init();
                let command = CommandState::init();
                AdaptiveCruiseControlService {
                    begin: std::time::Instant::now(),
                    context,
                    delayed,
                    input_store,
                    pid,
                    command,
                    output,
                    timer,
                }
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
                    let x_1 = (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.x_1.set(x_1);
                    let x = (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.x.set(x);
                } else {
                    let unique = self
                        .input_store
                        .speed_km_h
                        .replace((speed_km_h, _speed_km_h_instant));
                    assert!(unique.is_none(), "flow `speed_km_h` changes too frequently");
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
                        self.input_store.speed_km_h.take(),
                        self.input_store.distance_m.take(),
                    ) {
                        (None, None) => {}
                        (Some((speed_km_h, _speed_km_h_instant)), None) => {
                            self.context.speed_km_h.set(speed_km_h);
                            let x_1 =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x_1.set(x_1);
                            let x =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x.set(x);
                        }
                        (None, Some((distance_m, _distance_m_instant))) => {
                            self.context.distance_m.set(distance_m);
                            let x_1 =
                                (_distance_m_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x_1.set(x_1);
                            let x =
                                (_distance_m_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x.set(x);
                        }
                        (
                            Some((speed_km_h, _speed_km_h_instant)),
                            Some((distance_m, _distance_m_instant)),
                        ) => {
                            self.context.distance_m.set(distance_m);
                            self.context.speed_km_h.set(speed_km_h);
                            let x =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x.set(x);
                            let x_1 =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x_1.set(x_1);
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
            pub async fn handle_distance_m(
                &mut self,
                _distance_m_instant: std::time::Instant,
                distance_m: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_distance_m_instant).await?;
                    self.context.reset();
                    self.context.distance_m.set(distance_m);
                    let x_1 = (_distance_m_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.x_1.set(x_1);
                    let x = (_distance_m_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.x.set(x);
                } else {
                    let unique = self
                        .input_store
                        .distance_m
                        .replace((distance_m, _distance_m_instant));
                    assert!(unique.is_none(), "flow `distance_m` changes too frequently");
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
                let brakes_command_m_s = self.command.step(CommandInput {
                    distance_m: self.context.distance_m.get(),
                    sv_v_km_h: self.context.speed_km_h.get(),
                    t_ms: self.context.x.get(),
                });
                self.context.brakes_command_m_s.set(brakes_command_m_s);
                let brakes_m_s = self.pid.step(PidInput {
                    sv_v_km_h: self.context.speed_km_h.get(),
                    b_m_s_command: self.context.brakes_command_m_s.get(),
                    t_ms: self.context.x_1.get(),
                });
                self.context.brakes_m_s.set(brakes_m_s);
                self.send_output(O::BrakesMS(
                    self.context.brakes_m_s.get(),
                    _timeout_adaptive_cruise_control_instant,
                ))
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
            #[inline]
            pub async fn reset_time_constraints(
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
