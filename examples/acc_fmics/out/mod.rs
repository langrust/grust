#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Activation {
    #[default]
    On,
    Off,
}
pub fn safety_distance(sv_v: f64, fv_v: f64) -> f64 {
    let sv_d_stop = (sv_v * 1.0f64) + ((sv_v * sv_v) / (2.0f64 * 5.886f64));
    let fv_d_stop = (fv_v * fv_v) / (2.0f64 * 5.886f64);
    sv_d_stop - fv_d_stop
}
pub struct AccInput {
    pub c: bool,
    pub d: f64,
    pub v: f64,
    pub s: f64,
}
pub struct AccState {}
impl grust::core::Component for AccState {
    type Input = AccInput;
    type Output = f64;
    fn init() -> AccState {
        AccState {}
    }
    fn step(&mut self, input: AccInput) -> f64 {
        let (d_safe, b, fv_v) = match input.c {
            true => {
                let fv_v = input.s + input.v;
                let d_safe = safety_distance(input.s, fv_v);
                let b = (input.v * input.v) / (2.0f64 * (input.d - d_safe));
                (d_safe, b, fv_v)
            }
            false => {
                let b = 0.0f64;
                let (fv_v, d_safe) = (0.0f64, 0.0f64);
                (d_safe, b, fv_v)
            }
        };
        b
    }
}
pub struct ActivateInput {
    pub act: Option<Activation>,
    pub r: Option<f64>,
}
pub struct ActivateState {
    last_active: bool,
    last_approach: bool,
    last_d: f64,
}
impl grust::core::Component for ActivateState {
    type Input = ActivateInput;
    type Output = bool;
    fn init() -> ActivateState {
        ActivateState {
            last_active: false,
            last_approach: false,
            last_d: 0.0f64,
        }
    }
    fn step(&mut self, input: ActivateInput) -> bool {
        let (active, d, approach) = match (input.act, input.r) {
            (Some(act), _) => {
                let active = act == Activation::On;
                (active, self.last_d, self.last_approach)
            }
            (_, Some(r)) => {
                let d = r;
                let approach = d < self.last_d;
                (self.last_active, d, approach)
            }
            (_, _) => (self.last_active, self.last_d, self.last_approach),
        };
        let c = active && approach;
        self.last_active = active;
        self.last_approach = approach;
        self.last_d = d;
        c
    }
}
pub mod runtime {
    use super::*;
    use futures::{sink::SinkExt, stream::StreamExt};
    #[derive(PartialEq)]
    pub enum RuntimeTimer {
        DelayAdaptiveCruiseControl,
        TimeoutAdaptiveCruiseControl,
    }
    use RuntimeTimer as T;
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
        RadarM(f64, std::time::Instant),
        AccActive(Activation, std::time::Instant),
        SpeedKmH(f64, std::time::Instant),
        Timer(T, std::time::Instant),
    }
    use RuntimeInput as I;
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
                (I::RadarM(this, _), I::RadarM(other, _)) => this.eq(other),
                (I::AccActive(this, _), I::AccActive(other, _)) => this.eq(other),
                (I::SpeedKmH(this, _), I::SpeedKmH(other, _)) => this.eq(other),
                (I::Timer(this, _), I::Timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::RadarM(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::AccActive(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::SpeedKmH(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Timer(_, _grust_reserved_instant) => *_grust_reserved_instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    #[derive(Debug, PartialEq)]
    pub enum RuntimeOutput {
        BrakesMS(f64, std::time::Instant),
    }
    use RuntimeOutput as O;
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
            radar_m: f64,
            speed_km_h: f64,
        ) -> Result<(), futures::channel::mpsc::SendError> {
            futures::pin_mut!(input);
            let mut runtime = self;
            runtime
                .adaptive_cruise_control
                .handle_init(_grust_reserved_init_instant, radar_m, speed_km_h)
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
                    I::AccActive(acc_active, _grust_reserved_instant) => {
                        runtime
                            .adaptive_cruise_control
                            .handle_acc_active(_grust_reserved_instant, acc_active)
                            .await?;
                    }
                    I::RadarM(radar_m, _grust_reserved_instant) => {
                        runtime
                            .adaptive_cruise_control
                            .handle_radar_m(_grust_reserved_instant, radar_m)
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
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct RadarM(f64, bool);
            impl RadarM {
                pub fn set(&mut self, radar_m: f64) {
                    self.1 = self.0 != radar_m;
                    self.0 = radar_m;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct SpeedMS(f64, bool);
            impl SpeedMS {
                pub fn set(&mut self, speed_m_s: f64) {
                    self.1 = self.0 != speed_m_s;
                    self.0 = speed_m_s;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct VelDelta(f64, bool);
            impl VelDelta {
                pub fn set(&mut self, vel_delta: f64) {
                    self.1 = self.0 != vel_delta;
                    self.0 = vel_delta;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
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
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct RadarEOld(f64, bool);
            impl RadarEOld {
                pub fn set(&mut self, radar_e_old: f64) {
                    self.1 = self.0 != radar_e_old;
                    self.0 = radar_e_old;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
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
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct X(f64, bool);
            impl X {
                pub fn set(&mut self, x: f64) {
                    self.1 = self.0 != x;
                    self.0 = x;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct Cond(bool, bool);
            impl Cond {
                pub fn set(&mut self, cond: bool) {
                    self.1 = self.0 != cond;
                    self.0 = cond;
                }
                pub fn get(&self) -> bool {
                    self.0
                }
                pub fn take(&mut self) -> bool {
                    std::mem::take(&mut self.0)
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
            pub radar_m: ctx_ty::RadarM,
            pub speed_m_s: ctx_ty::SpeedMS,
            pub vel_delta: ctx_ty::VelDelta,
            pub brakes_m_s: ctx_ty::BrakesMS,
            pub radar_e_old: ctx_ty::RadarEOld,
            pub speed_km_h: ctx_ty::SpeedKmH,
            pub x: ctx_ty::X,
            pub cond: ctx_ty::Cond,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.radar_m.reset();
                self.speed_m_s.reset();
                self.vel_delta.reset();
                self.brakes_m_s.reset();
                self.radar_e_old.reset();
                self.speed_km_h.reset();
                self.x.reset();
                self.cond.reset();
            }
        }
        #[derive(Default)]
        pub struct AdaptiveCruiseControlServiceStore {
            radar_m: Option<(f64, std::time::Instant)>,
            acc_active: Option<(Activation, std::time::Instant)>,
            speed_km_h: Option<(f64, std::time::Instant)>,
        }
        impl AdaptiveCruiseControlServiceStore {
            pub fn not_empty(&self) -> bool {
                self.radar_m.is_some() || self.acc_active.is_some() || self.speed_km_h.is_some()
            }
        }
        pub struct AdaptiveCruiseControlService {
            begin: std::time::Instant,
            context: Context,
            delayed: bool,
            input_store: AdaptiveCruiseControlServiceStore,
            acc: AccState,
            derive: grust::std::time::derivation::DeriveState,
            activate: ActivateState,
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
                let acc = <AccState as grust::core::Component>::init();
                let derive =
                    <grust::std::time::derivation::DeriveState as grust::core::Component>::init();
                let activate = <ActivateState as grust::core::Component>::init();
                AdaptiveCruiseControlService {
                    begin: std::time::Instant::now(),
                    context,
                    delayed,
                    input_store,
                    acc,
                    derive,
                    activate,
                    output,
                    timer,
                }
            }
            pub async fn handle_init(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
                radar_m: f64,
                speed_km_h: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_timeout(_grust_reserved_instant).await?;
                self.context.speed_km_h.set(speed_km_h);
                let speed_m_s = utils::convert(speed_km_h);
                self.context.speed_m_s.set(speed_m_s);
                self.context.radar_m.set(radar_m);
                self.context.radar_e_old.set(radar_m);
                let cond = <ActivateState as grust::core::Component>::step(
                    &mut self.activate,
                    ActivateInput { act: None, r: None },
                );
                self.context.cond.set(cond);
                let x = (_grust_reserved_instant
                    .duration_since(self.begin)
                    .as_millis()) as f64;
                self.context.x.set(x);
                let vel_delta =
                    <grust::std::time::derivation::DeriveState as grust::core::Component>::step(
                        &mut self.derive,
                        grust::std::time::derivation::DeriveInput { x: radar_m, t: x },
                    );
                self.context.vel_delta.set(vel_delta);
                let brakes_m_s = <AccState as grust::core::Component>::step(
                    &mut self.acc,
                    AccInput {
                        c: self.context.cond.get(),
                        d: radar_m,
                        v: self.context.vel_delta.get(),
                        s: self.context.speed_m_s.get(),
                    },
                );
                self.context.brakes_m_s.set(brakes_m_s);
                self.send_output(
                    O::BrakesMS(self.context.brakes_m_s.get(), _grust_reserved_instant),
                    _grust_reserved_instant,
                )
                .await?;
                Ok(())
            }
            pub async fn handle_radar_m(
                &mut self,
                _radar_m_instant: std::time::Instant,
                radar_m: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_radar_m_instant).await?;
                    self.context.reset();
                    let radar_e_ref = &mut None;
                    self.context.radar_m.set(radar_m);
                    if self.context.radar_e_old.get() != radar_m {
                        self.context.radar_e_old.set(radar_m);
                        *radar_e_ref = Some(radar_m);
                    }
                    if radar_e_ref.is_some() {
                        let cond = <ActivateState as grust::core::Component>::step(
                            &mut self.activate,
                            ActivateInput {
                                act: None,
                                r: *radar_e_ref,
                            },
                        );
                        self.context.cond.set(cond);
                    }
                    let x = (_radar_m_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.x.set(x);
                    if self.context.radar_m.is_new() || self.context.x.is_new() {
                        let vel_delta = < grust :: std :: time :: derivation :: DeriveState as grust :: core :: Component > :: step (& mut self . derive , grust :: std :: time :: derivation :: DeriveInput {x : radar_m , t : x}) ;
                        self.context.vel_delta.set(vel_delta);
                    }
                    if self.context.cond.is_new()
                        || self.context.radar_m.is_new()
                        || self.context.vel_delta.is_new()
                        || self.context.speed_m_s.is_new()
                    {
                        let brakes_m_s = <AccState as grust::core::Component>::step(
                            &mut self.acc,
                            AccInput {
                                c: self.context.cond.get(),
                                d: radar_m,
                                v: self.context.vel_delta.get(),
                                s: self.context.speed_m_s.get(),
                            },
                        );
                        self.context.brakes_m_s.set(brakes_m_s);
                    }
                    if self.context.brakes_m_s.is_new() {
                        self.send_output(
                            O::BrakesMS(self.context.brakes_m_s.get(), _radar_m_instant),
                            _radar_m_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .radar_m
                        .replace((radar_m, _radar_m_instant));
                    assert!(unique.is_none(), "flow `radar_m` changes too frequently");
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
                        self.input_store.radar_m.take(),
                        self.input_store.acc_active.take(),
                        self.input_store.speed_km_h.take(),
                    ) {
                        (None, None, None) => {}
                        (Some((radar_m, _radar_m_instant)), None, None) => {
                            let radar_e_ref = &mut None;
                            self.context.radar_m.set(radar_m);
                            if self.context.radar_e_old.get() != radar_m {
                                self.context.radar_e_old.set(radar_m);
                                *radar_e_ref = Some(radar_m);
                            }
                            if radar_e_ref.is_some() {
                                let cond = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        act: None,
                                        r: *radar_e_ref,
                                    },
                                );
                                self.context.cond.set(cond);
                            }
                            let x = (_grust_reserved_instant
                                .duration_since(self.begin)
                                .as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.radar_m.is_new() || self.context.x.is_new() {
                                let vel_delta = < grust :: std :: time :: derivation :: DeriveState as grust :: core :: Component > :: step (& mut self . derive , grust :: std :: time :: derivation :: DeriveInput {x : radar_m , t : x}) ;
                                self.context.vel_delta.set(vel_delta);
                            }
                            if self.context.cond.is_new()
                                || self.context.radar_m.is_new()
                                || self.context.vel_delta.is_new()
                                || self.context.speed_m_s.is_new()
                            {
                                let brakes_m_s = <AccState as grust::core::Component>::step(
                                    &mut self.acc,
                                    AccInput {
                                        c: self.context.cond.get(),
                                        d: radar_m,
                                        v: self.context.vel_delta.get(),
                                        s: self.context.speed_m_s.get(),
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
                        (None, Some((acc_active, _acc_active_instant)), None) => {
                            let acc_active_ref = &mut None;
                            *acc_active_ref = Some(acc_active);
                            if acc_active_ref.is_some() {
                                let cond = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        act: *acc_active_ref,
                                        r: None,
                                    },
                                );
                                self.context.cond.set(cond);
                            }
                            let x = (_grust_reserved_instant
                                .duration_since(self.begin)
                                .as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.radar_m.is_new() || self.context.x.is_new() {
                                let vel_delta = < grust :: std :: time :: derivation :: DeriveState as grust :: core :: Component > :: step (& mut self . derive , grust :: std :: time :: derivation :: DeriveInput {x : self . context . radar_m . get () , t : x}) ;
                                self.context.vel_delta.set(vel_delta);
                            }
                            if self.context.cond.is_new()
                                || self.context.radar_m.is_new()
                                || self.context.vel_delta.is_new()
                                || self.context.speed_m_s.is_new()
                            {
                                let brakes_m_s = <AccState as grust::core::Component>::step(
                                    &mut self.acc,
                                    AccInput {
                                        c: self.context.cond.get(),
                                        d: self.context.radar_m.get(),
                                        v: self.context.vel_delta.get(),
                                        s: self.context.speed_m_s.get(),
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
                            Some((radar_m, _radar_m_instant)),
                            Some((acc_active, _acc_active_instant)),
                            None,
                        ) => {
                            let acc_active_ref = &mut None;
                            let radar_e_ref = &mut None;
                            *acc_active_ref = Some(acc_active);
                            self.context.radar_m.set(radar_m);
                            if self.context.radar_e_old.get() != radar_m {
                                self.context.radar_e_old.set(radar_m);
                                *radar_e_ref = Some(radar_m);
                            }
                            if acc_active_ref.is_some() || radar_e_ref.is_some() {
                                let cond = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        act: *acc_active_ref,
                                        r: *radar_e_ref,
                                    },
                                );
                                self.context.cond.set(cond);
                            }
                            let x = (_grust_reserved_instant
                                .duration_since(self.begin)
                                .as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.radar_m.is_new() || self.context.x.is_new() {
                                let vel_delta = < grust :: std :: time :: derivation :: DeriveState as grust :: core :: Component > :: step (& mut self . derive , grust :: std :: time :: derivation :: DeriveInput {x : radar_m , t : x}) ;
                                self.context.vel_delta.set(vel_delta);
                            }
                            if self.context.cond.is_new()
                                || self.context.radar_m.is_new()
                                || self.context.vel_delta.is_new()
                                || self.context.speed_m_s.is_new()
                            {
                                let brakes_m_s = <AccState as grust::core::Component>::step(
                                    &mut self.acc,
                                    AccInput {
                                        c: self.context.cond.get(),
                                        d: radar_m,
                                        v: self.context.vel_delta.get(),
                                        s: self.context.speed_m_s.get(),
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
                        (None, None, Some((speed_km_h, _speed_km_h_instant))) => {
                            self.context.speed_km_h.set(speed_km_h);
                            if self.context.speed_km_h.is_new() {
                                let speed_m_s = utils::convert(speed_km_h);
                                self.context.speed_m_s.set(speed_m_s);
                            }
                            let x = (_grust_reserved_instant
                                .duration_since(self.begin)
                                .as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.radar_m.is_new() || self.context.x.is_new() {
                                let vel_delta = < grust :: std :: time :: derivation :: DeriveState as grust :: core :: Component > :: step (& mut self . derive , grust :: std :: time :: derivation :: DeriveInput {x : self . context . radar_m . get () , t : x}) ;
                                self.context.vel_delta.set(vel_delta);
                            }
                            if self.context.cond.is_new()
                                || self.context.radar_m.is_new()
                                || self.context.vel_delta.is_new()
                                || self.context.speed_m_s.is_new()
                            {
                                let brakes_m_s = <AccState as grust::core::Component>::step(
                                    &mut self.acc,
                                    AccInput {
                                        c: self.context.cond.get(),
                                        d: self.context.radar_m.get(),
                                        v: self.context.vel_delta.get(),
                                        s: self.context.speed_m_s.get(),
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
                            Some((radar_m, _radar_m_instant)),
                            None,
                            Some((speed_km_h, _speed_km_h_instant)),
                        ) => {
                            let radar_e_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            if self.context.speed_km_h.is_new() {
                                let speed_m_s = utils::convert(speed_km_h);
                                self.context.speed_m_s.set(speed_m_s);
                            }
                            self.context.radar_m.set(radar_m);
                            if self.context.radar_e_old.get() != radar_m {
                                self.context.radar_e_old.set(radar_m);
                                *radar_e_ref = Some(radar_m);
                            }
                            if radar_e_ref.is_some() {
                                let cond = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        act: None,
                                        r: *radar_e_ref,
                                    },
                                );
                                self.context.cond.set(cond);
                            }
                            let x = (_grust_reserved_instant
                                .duration_since(self.begin)
                                .as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.radar_m.is_new() || self.context.x.is_new() {
                                let vel_delta = < grust :: std :: time :: derivation :: DeriveState as grust :: core :: Component > :: step (& mut self . derive , grust :: std :: time :: derivation :: DeriveInput {x : radar_m , t : x}) ;
                                self.context.vel_delta.set(vel_delta);
                            }
                            if self.context.cond.is_new()
                                || self.context.radar_m.is_new()
                                || self.context.vel_delta.is_new()
                                || self.context.speed_m_s.is_new()
                            {
                                let brakes_m_s = <AccState as grust::core::Component>::step(
                                    &mut self.acc,
                                    AccInput {
                                        c: self.context.cond.get(),
                                        d: radar_m,
                                        v: self.context.vel_delta.get(),
                                        s: self.context.speed_m_s.get(),
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
                            Some((acc_active, _acc_active_instant)),
                            Some((speed_km_h, _speed_km_h_instant)),
                        ) => {
                            let acc_active_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            if self.context.speed_km_h.is_new() {
                                let speed_m_s = utils::convert(speed_km_h);
                                self.context.speed_m_s.set(speed_m_s);
                            }
                            *acc_active_ref = Some(acc_active);
                            if acc_active_ref.is_some() {
                                let cond = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        act: *acc_active_ref,
                                        r: None,
                                    },
                                );
                                self.context.cond.set(cond);
                            }
                            let x = (_grust_reserved_instant
                                .duration_since(self.begin)
                                .as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.radar_m.is_new() || self.context.x.is_new() {
                                let vel_delta = < grust :: std :: time :: derivation :: DeriveState as grust :: core :: Component > :: step (& mut self . derive , grust :: std :: time :: derivation :: DeriveInput {x : self . context . radar_m . get () , t : x}) ;
                                self.context.vel_delta.set(vel_delta);
                            }
                            if self.context.cond.is_new()
                                || self.context.radar_m.is_new()
                                || self.context.vel_delta.is_new()
                                || self.context.speed_m_s.is_new()
                            {
                                let brakes_m_s = <AccState as grust::core::Component>::step(
                                    &mut self.acc,
                                    AccInput {
                                        c: self.context.cond.get(),
                                        d: self.context.radar_m.get(),
                                        v: self.context.vel_delta.get(),
                                        s: self.context.speed_m_s.get(),
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
                            Some((radar_m, _radar_m_instant)),
                            Some((acc_active, _acc_active_instant)),
                            Some((speed_km_h, _speed_km_h_instant)),
                        ) => {
                            let acc_active_ref = &mut None;
                            let radar_e_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            if self.context.speed_km_h.is_new() {
                                let speed_m_s = utils::convert(speed_km_h);
                                self.context.speed_m_s.set(speed_m_s);
                            }
                            *acc_active_ref = Some(acc_active);
                            self.context.radar_m.set(radar_m);
                            if self.context.radar_e_old.get() != radar_m {
                                self.context.radar_e_old.set(radar_m);
                                *radar_e_ref = Some(radar_m);
                            }
                            if acc_active_ref.is_some() || radar_e_ref.is_some() {
                                let cond = <ActivateState as grust::core::Component>::step(
                                    &mut self.activate,
                                    ActivateInput {
                                        act: *acc_active_ref,
                                        r: *radar_e_ref,
                                    },
                                );
                                self.context.cond.set(cond);
                            }
                            let x = (_grust_reserved_instant
                                .duration_since(self.begin)
                                .as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.radar_m.is_new() || self.context.x.is_new() {
                                let vel_delta = < grust :: std :: time :: derivation :: DeriveState as grust :: core :: Component > :: step (& mut self . derive , grust :: std :: time :: derivation :: DeriveInput {x : radar_m , t : x}) ;
                                self.context.vel_delta.set(vel_delta);
                            }
                            if self.context.cond.is_new()
                                || self.context.radar_m.is_new()
                                || self.context.vel_delta.is_new()
                                || self.context.speed_m_s.is_new()
                            {
                                let brakes_m_s = <AccState as grust::core::Component>::step(
                                    &mut self.acc,
                                    AccInput {
                                        c: self.context.cond.get(),
                                        d: radar_m,
                                        v: self.context.vel_delta.get(),
                                        s: self.context.speed_m_s.get(),
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
                    if acc_active_ref.is_some() {
                        let cond = <ActivateState as grust::core::Component>::step(
                            &mut self.activate,
                            ActivateInput {
                                act: *acc_active_ref,
                                r: None,
                            },
                        );
                        self.context.cond.set(cond);
                    }
                    let x = (_acc_active_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.x.set(x);
                    if self.context.radar_m.is_new() || self.context.x.is_new() {
                        let vel_delta = < grust :: std :: time :: derivation :: DeriveState as grust :: core :: Component > :: step (& mut self . derive , grust :: std :: time :: derivation :: DeriveInput {x : self . context . radar_m . get () , t : x}) ;
                        self.context.vel_delta.set(vel_delta);
                    }
                    if self.context.cond.is_new()
                        || self.context.radar_m.is_new()
                        || self.context.vel_delta.is_new()
                        || self.context.speed_m_s.is_new()
                    {
                        let brakes_m_s = <AccState as grust::core::Component>::step(
                            &mut self.acc,
                            AccInput {
                                c: self.context.cond.get(),
                                d: self.context.radar_m.get(),
                                v: self.context.vel_delta.get(),
                                s: self.context.speed_m_s.get(),
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
            pub async fn handle_timeout_adaptive_cruise_control(
                &mut self,
                _timeout_adaptive_cruise_control_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constraints(_timeout_adaptive_cruise_control_instant)
                    .await?;
                self.context.reset();
                let x = (_timeout_adaptive_cruise_control_instant
                    .duration_since(self.begin)
                    .as_millis()) as f64;
                self.context.x.set(x);
                if self.context.radar_m.is_new() || self.context.x.is_new() {
                    let vel_delta =
                        <grust::std::time::derivation::DeriveState as grust::core::Component>::step(
                            &mut self.derive,
                            grust::std::time::derivation::DeriveInput {
                                x: self.context.radar_m.get(),
                                t: x,
                            },
                        );
                    self.context.vel_delta.set(vel_delta);
                }
                if self.context.cond.is_new()
                    || self.context.radar_m.is_new()
                    || self.context.vel_delta.is_new()
                    || self.context.speed_m_s.is_new()
                {
                    let brakes_m_s = <AccState as grust::core::Component>::step(
                        &mut self.acc,
                        AccInput {
                            c: self.context.cond.get(),
                            d: self.context.radar_m.get(),
                            v: self.context.vel_delta.get(),
                            s: self.context.speed_m_s.get(),
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
            pub async fn handle_speed_km_h(
                &mut self,
                _speed_km_h_instant: std::time::Instant,
                speed_km_h: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_speed_km_h_instant).await?;
                    self.context.reset();
                    self.context.speed_km_h.set(speed_km_h);
                    if self.context.speed_km_h.is_new() {
                        let speed_m_s = utils::convert(speed_km_h);
                        self.context.speed_m_s.set(speed_m_s);
                    }
                    let x = (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.x.set(x);
                    if self.context.radar_m.is_new() || self.context.x.is_new() {
                        let vel_delta = < grust :: std :: time :: derivation :: DeriveState as grust :: core :: Component > :: step (& mut self . derive , grust :: std :: time :: derivation :: DeriveInput {x : self . context . radar_m . get () , t : x}) ;
                        self.context.vel_delta.set(vel_delta);
                    }
                    if self.context.cond.is_new()
                        || self.context.radar_m.is_new()
                        || self.context.vel_delta.is_new()
                        || self.context.speed_m_s.is_new()
                    {
                        let brakes_m_s = <AccState as grust::core::Component>::step(
                            &mut self.acc,
                            AccInput {
                                c: self.context.cond.get(),
                                d: self.context.radar_m.get(),
                                v: self.context.vel_delta.get(),
                                s: self.context.speed_m_s.get(),
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
                self.output.feed(output).await?;
                Ok(())
            }
            #[inline]
            pub async fn send_timer(
                &mut self,
                timer: T,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer.feed((timer, instant)).await?;
                Ok(())
            }
        }
    }
}
