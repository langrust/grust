#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Braking {
    #[default]
    NoBrake,
    SoftBrake,
    UrgentBrake,
}
pub fn compute_soft_braking_distance(speed: f64, acc: f64) -> f64 {
    (speed * speed) / (100.0f64 * acc)
}
pub fn brakes(distance: f64, speed: f64, acc: f64) -> Braking {
    let braking_distance = compute_soft_braking_distance(speed, acc);
    let response = if braking_distance < distance {
        Braking::SoftBrake
    } else {
        Braking::UrgentBrake
    };
    response
}
pub struct DeriveInput {
    pub v_km_h: f64,
    pub t: f64,
}
pub struct DeriveOutput {
    pub a_km_h: f64,
}
pub struct DeriveState {
    last_a: f64,
    last_t: f64,
    last_v: f64,
    last_x: bool,
}
impl grust::core::Component for DeriveState {
    type Input = DeriveInput;
    type Output = DeriveOutput;
    fn init() -> DeriveState {
        DeriveState {
            last_a: 0.0f64,
            last_t: 0.0f64,
            last_v: 0.0f64,
            last_x: false,
        }
    }
    fn step(&mut self, input: DeriveInput) -> DeriveOutput {
        let v = input.v_km_h / 3.6f64;
        let dt = input.t - self.last_t;
        let x = dt > 10.0f64;
        let a = match () {
            () if x && !(self.last_x) => (v - self.last_v) / dt,
            () => self.last_a,
        };
        let a_km_h = 3.6f64 * a;
        self.last_a = a;
        self.last_t = input.t;
        self.last_v = v;
        self.last_x = x;
        DeriveOutput { a_km_h }
    }
}
pub struct BrakingStateInput {
    pub pedest: Option<f64>,
    pub timeout_pedest: Option<()>,
    pub speed: f64,
    pub acc: f64,
}
pub struct BrakingStateOutput {
    pub state: Braking,
}
pub struct BrakingStateState {
    last_state: Braking,
}
impl grust::core::Component for BrakingStateState {
    type Input = BrakingStateInput;
    type Output = BrakingStateOutput;
    fn init() -> BrakingStateState {
        BrakingStateState {
            last_state: Braking::NoBrake,
        }
    }
    fn step(&mut self, input: BrakingStateInput) -> BrakingStateOutput {
        let state = match (input.pedest, input.timeout_pedest) {
            (Some(d), _) => {
                let state = brakes(d, input.speed, input.acc);
                state
            }
            (_, Some(_)) => {
                let state = Braking::NoBrake;
                state
            }
            (_, _) => self.last_state,
        };
        self.last_state = state;
        BrakingStateOutput { state }
    }
}
pub mod runtime {
    use super::*;
    use grust::futures::{sink::SinkExt, stream::StreamExt};
    #[derive(Debug)]
    pub enum RuntimeInput {
        SpeedKmH(f64, std::time::Instant),
        PedestrianL(f64, std::time::Instant),
        PedestrianR(f64, std::time::Instant),
        Timer(T, std::time::Instant),
    }
    use RuntimeInput as I;
    impl grust::core::priority_stream::Reset for RuntimeInput {
        fn do_reset(&self) -> bool {
            match self {
                I::Timer(timer, _) => grust::core::timer_stream::Timing::do_reset(timer),
                _ => false,
            }
        }
    }
    impl PartialEq for RuntimeInput {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (I::SpeedKmH(this, _), I::SpeedKmH(other, _)) => this.eq(other),
                (I::PedestrianL(this, _), I::PedestrianL(other, _)) => this.eq(other),
                (I::PedestrianR(this, _), I::PedestrianR(other, _)) => this.eq(other),
                (I::Timer(this, _), I::Timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::SpeedKmH(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::PedestrianL(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::PedestrianR(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Timer(_, _grust_reserved_instant) => *_grust_reserved_instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    #[derive(Debug, PartialEq)]
    pub enum RuntimeOutput {
        Brakes(Braking, std::time::Instant),
    }
    use RuntimeOutput as O;
    #[derive(Debug, Default)]
    pub struct RuntimeInit {}
    #[derive(Debug, PartialEq)]
    pub enum RuntimeTimer {
        TimeoutTimeoutPedest,
        DelayAeb,
        TimeoutAeb,
    }
    use RuntimeTimer as T;
    impl grust::core::timer_stream::Timing for RuntimeTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::TimeoutTimeoutPedest => std::time::Duration::from_millis(2000u64),
                T::DelayAeb => std::time::Duration::from_millis(10u64),
                T::TimeoutAeb => std::time::Duration::from_millis(3000u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::TimeoutTimeoutPedest => true,
                T::DelayAeb => true,
                T::TimeoutAeb => true,
            }
        }
    }
    pub struct Runtime {
        aeb: aeb_service::AebService,
        output: grust::futures::channel::mpsc::Sender<O>,
        timer: grust::futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl Runtime {
        pub fn new(
            output: grust::futures::channel::mpsc::Sender<O>,
            timer: grust::futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let aeb = aeb_service::AebService::init(output.clone(), timer.clone());
            Runtime { aeb, output, timer }
        }
        #[inline]
        pub async fn send_timer(
            &mut self,
            timer: T,
            instant: std::time::Instant,
        ) -> Result<(), grust::futures::channel::mpsc::SendError> {
            self.timer.send((timer, instant)).await?;
            Ok(())
        }
        pub async fn run_loop(
            self,
            _grust_reserved_init_instant: std::time::Instant,
            input: impl grust::futures::Stream<Item = I>,
            init_vals: RuntimeInit,
        ) -> Result<(), grust::futures::channel::mpsc::SendError> {
            grust::futures::pin_mut!(input);
            let mut runtime = self;
            let RuntimeInit {} = init_vals;
            runtime
                .aeb
                .handle_init(_grust_reserved_init_instant)
                .await?;
            while let Some(input) = input.next().await {
                match input {
                    I::PedestrianL(pedestrian_l, _grust_reserved_instant) => {
                        runtime
                            .aeb
                            .handle_pedestrian_l(_grust_reserved_instant, pedestrian_l)
                            .await?;
                    }
                    I::PedestrianR(pedestrian_r, _grust_reserved_instant) => {
                        runtime
                            .aeb
                            .handle_pedestrian_r(_grust_reserved_instant, pedestrian_r)
                            .await?;
                    }
                    I::SpeedKmH(speed_km_h, _grust_reserved_instant) => {
                        runtime
                            .aeb
                            .handle_speed_km_h(_grust_reserved_instant, speed_km_h)
                            .await?;
                    }
                    I::Timer(T::DelayAeb, _grust_reserved_instant) => {
                        runtime
                            .aeb
                            .handle_delay_aeb(_grust_reserved_instant)
                            .await?;
                    }
                    I::Timer(T::TimeoutAeb, _grust_reserved_instant) => {
                        runtime
                            .aeb
                            .handle_timeout_aeb(_grust_reserved_instant)
                            .await?;
                    }
                    I::Timer(T::TimeoutTimeoutPedest, _grust_reserved_instant) => {
                        runtime
                            .aeb
                            .handle_timeout_timeout_pedest(_grust_reserved_instant)
                            .await?;
                    }
                }
            }
            Ok(())
        }
    }
    pub mod aeb_service {
        use super::*;
        use grust::futures::{sink::SinkExt, stream::StreamExt};
        mod ctx_ty {
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct Brakes(super::Braking, bool);
            impl Brakes {
                pub fn set(&mut self, brakes: super::Braking) {
                    self.1 = self.0 != brakes;
                    self.0 = brakes;
                }
                pub fn get(&self) -> super::Braking {
                    self.0
                }
                pub fn take(&mut self) -> super::Braking {
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
            pub struct SpeedKmHBis(f64, bool);
            impl SpeedKmHBis {
                pub fn set(&mut self, speed_km_h_bis: f64) {
                    self.1 = self.0 != speed_km_h_bis;
                    self.0 = speed_km_h_bis;
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
            pub struct AccKmH(f64, bool);
            impl AccKmH {
                pub fn set(&mut self, acc_km_h: f64) {
                    self.1 = self.0 != acc_km_h;
                    self.0 = acc_km_h;
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
        }
        #[derive(Clone, Copy, PartialEq, Default, Debug)]
        pub struct Context {
            pub brakes: ctx_ty::Brakes,
            pub speed_km_h_bis: ctx_ty::SpeedKmHBis,
            pub x: ctx_ty::X,
            pub acc_km_h: ctx_ty::AccKmH,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.brakes.reset();
                self.speed_km_h_bis.reset();
                self.x.reset();
                self.acc_km_h.reset();
            }
        }
        #[derive(Default)]
        pub struct AebServiceStore {
            speed_km_h: Option<(f64, std::time::Instant)>,
            pedestrian_l: Option<(f64, std::time::Instant)>,
            timeout_timeout_pedest: Option<((), std::time::Instant)>,
            pedestrian_r: Option<(f64, std::time::Instant)>,
        }
        impl AebServiceStore {
            pub fn not_empty(&self) -> bool {
                self.speed_km_h.is_some()
                    || self.pedestrian_l.is_some()
                    || self.timeout_timeout_pedest.is_some()
                    || self.pedestrian_r.is_some()
            }
        }
        pub struct AebService {
            begin: std::time::Instant,
            context: Context,
            delayed: bool,
            input_store: AebServiceStore,
            derive: DeriveState,
            braking_state: BrakingStateState,
            output: grust::futures::channel::mpsc::Sender<O>,
            timer: grust::futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        }
        impl AebService {
            pub fn init(
                output: grust::futures::channel::mpsc::Sender<O>,
                timer: grust::futures::channel::mpsc::Sender<(T, std::time::Instant)>,
            ) -> AebService {
                let context = Context::init();
                let delayed = true;
                let input_store = Default::default();
                let derive = <DeriveState as grust::core::Component>::init();
                let braking_state = <BrakingStateState as grust::core::Component>::init();
                AebService {
                    begin: std::time::Instant::now(),
                    context,
                    delayed,
                    input_store,
                    derive,
                    braking_state,
                    output,
                    timer,
                }
            }
            pub async fn handle_init(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                self.reset_service_timeout(_grust_reserved_instant).await?;
                self.send_timer(T::TimeoutTimeoutPedest, _grust_reserved_instant)
                    .await?;
                let x = (_grust_reserved_instant
                    .duration_since(self.begin)
                    .as_millis()) as f64;
                self.context.x.set(x);
                let DeriveOutput { a_km_h: acc_km_h } =
                    <DeriveState as grust::core::Component>::step(
                        &mut self.derive,
                        DeriveInput {
                            v_km_h: self.context.speed_km_h_bis.get(),
                            t: x,
                        },
                    );
                self.context.acc_km_h.set(acc_km_h);
                let BrakingStateOutput { state: brakes } =
                    <BrakingStateState as grust::core::Component>::step(
                        &mut self.braking_state,
                        BrakingStateInput {
                            pedest: None,
                            timeout_pedest: None,
                            speed: self.context.speed_km_h_bis.get(),
                            acc: self.context.acc_km_h.get(),
                        },
                    );
                self.context.brakes.set(brakes);
                self.send_output(
                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                    _grust_reserved_instant,
                )
                .await?;
                Ok(())
            }
            pub async fn handle_speed_km_h(
                &mut self,
                _speed_km_h_instant: std::time::Instant,
                speed_km_h: f64,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_speed_km_h_instant).await?;
                    self.context.reset();
                    let speed_km_h_ref = &mut None;
                    *speed_km_h_ref = Some(speed_km_h);
                    if self.context.speed_km_h_bis.get() != *speed_km_h_ref {
                        self.context.speed_km_h_bis.set(*speed_km_h_ref);
                    }
                    let x = (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.x.set(x);
                    if self.context.speed_km_h_bis.is_new() || self.context.x.is_new() {
                        let DeriveOutput { a_km_h: acc_km_h } =
                            <DeriveState as grust::core::Component>::step(
                                &mut self.derive,
                                DeriveInput {
                                    v_km_h: self.context.speed_km_h_bis.get(),
                                    t: x,
                                },
                            );
                        self.context.acc_km_h.set(acc_km_h);
                    }
                    if self.context.speed_km_h_bis.is_new() || self.context.acc_km_h.is_new() {
                        let BrakingStateOutput { state: brakes } =
                            <BrakingStateState as grust::core::Component>::step(
                                &mut self.braking_state,
                                BrakingStateInput {
                                    pedest: None,
                                    timeout_pedest: None,
                                    speed: self.context.speed_km_h_bis.get(),
                                    acc: self.context.acc_km_h.get(),
                                },
                            );
                        self.context.brakes.set(brakes);
                    }
                    if self.context.brakes.is_new() {
                        self.send_output(
                            O::Brakes(self.context.brakes.get(), _speed_km_h_instant),
                            _speed_km_h_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .speed_km_h
                        .replace((speed_km_h, _speed_km_h_instant));
                    assert ! (unique . is_none () , "flow `speed_km_h` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_pedestrian_l(
                &mut self,
                _pedestrian_l_instant: std::time::Instant,
                pedestrian_l: f64,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_pedestrian_l_instant).await?;
                    self.context.reset();
                    let pedestrian_ref = &mut None;
                    let pedestrian_l_ref = &mut None;
                    *pedestrian_l_ref = Some(pedestrian_l);
                    if pedestrian_l_ref.is_some() {
                        *pedestrian_ref = *pedestrian_l_ref;
                    }
                    if pedestrian_ref.is_some() {
                        self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                            .await?;
                    }
                    let x = (_pedestrian_l_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.x.set(x);
                    if self.context.speed_km_h_bis.is_new() || self.context.x.is_new() {
                        let DeriveOutput { a_km_h: acc_km_h } =
                            <DeriveState as grust::core::Component>::step(
                                &mut self.derive,
                                DeriveInput {
                                    v_km_h: self.context.speed_km_h_bis.get(),
                                    t: x,
                                },
                            );
                        self.context.acc_km_h.set(acc_km_h);
                    }
                    if pedestrian_ref.is_some()
                        || self.context.speed_km_h_bis.is_new()
                        || self.context.acc_km_h.is_new()
                    {
                        let BrakingStateOutput { state: brakes } =
                            <BrakingStateState as grust::core::Component>::step(
                                &mut self.braking_state,
                                BrakingStateInput {
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: None,
                                    speed: self.context.speed_km_h_bis.get(),
                                    acc: self.context.acc_km_h.get(),
                                },
                            );
                        self.context.brakes.set(brakes);
                    }
                    if self.context.brakes.is_new() {
                        self.send_output(
                            O::Brakes(self.context.brakes.get(), _pedestrian_l_instant),
                            _pedestrian_l_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .pedestrian_l
                        .replace((pedestrian_l, _pedestrian_l_instant));
                    assert ! (unique . is_none () , "flow `pedestrian_l` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_timeout_timeout_pedest(
                &mut self,
                _timeout_timeout_pedest_instant: std::time::Instant,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_timeout_timeout_pedest_instant)
                        .await?;
                    self.context.reset();
                    let timeout_pedest_ref = &mut None;
                    *timeout_pedest_ref = Some(());
                    self.send_timer(T::TimeoutTimeoutPedest, _timeout_timeout_pedest_instant)
                        .await?;
                    let x = (_timeout_timeout_pedest_instant
                        .duration_since(self.begin)
                        .as_millis()) as f64;
                    self.context.x.set(x);
                    if self.context.speed_km_h_bis.is_new() || self.context.x.is_new() {
                        let DeriveOutput { a_km_h: acc_km_h } =
                            <DeriveState as grust::core::Component>::step(
                                &mut self.derive,
                                DeriveInput {
                                    v_km_h: self.context.speed_km_h_bis.get(),
                                    t: x,
                                },
                            );
                        self.context.acc_km_h.set(acc_km_h);
                    }
                    if timeout_pedest_ref.is_some()
                        || self.context.speed_km_h_bis.is_new()
                        || self.context.acc_km_h.is_new()
                    {
                        let BrakingStateOutput { state: brakes } =
                            <BrakingStateState as grust::core::Component>::step(
                                &mut self.braking_state,
                                BrakingStateInput {
                                    pedest: None,
                                    timeout_pedest: *timeout_pedest_ref,
                                    speed: self.context.speed_km_h_bis.get(),
                                    acc: self.context.acc_km_h.get(),
                                },
                            );
                        self.context.brakes.set(brakes);
                    }
                    if self.context.brakes.is_new() {
                        self.send_output(
                            O::Brakes(self.context.brakes.get(), _timeout_timeout_pedest_instant),
                            _timeout_timeout_pedest_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .timeout_timeout_pedest
                        .replace(((), _timeout_timeout_pedest_instant));
                    assert ! (unique . is_none () , "flow `timeout_timeout_pedest` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_pedestrian_r(
                &mut self,
                _pedestrian_r_instant: std::time::Instant,
                pedestrian_r: f64,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_pedestrian_r_instant).await?;
                    self.context.reset();
                    let pedestrian_r_ref = &mut None;
                    let pedestrian_ref = &mut None;
                    *pedestrian_r_ref = Some(pedestrian_r);
                    if pedestrian_r_ref.is_some() {
                        *pedestrian_ref = *pedestrian_r_ref;
                    }
                    if pedestrian_ref.is_some() {
                        self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                            .await?;
                    }
                    let x = (_pedestrian_r_instant.duration_since(self.begin).as_millis()) as f64;
                    self.context.x.set(x);
                    if self.context.speed_km_h_bis.is_new() || self.context.x.is_new() {
                        let DeriveOutput { a_km_h: acc_km_h } =
                            <DeriveState as grust::core::Component>::step(
                                &mut self.derive,
                                DeriveInput {
                                    v_km_h: self.context.speed_km_h_bis.get(),
                                    t: x,
                                },
                            );
                        self.context.acc_km_h.set(acc_km_h);
                    }
                    if pedestrian_ref.is_some()
                        || self.context.speed_km_h_bis.is_new()
                        || self.context.acc_km_h.is_new()
                    {
                        let BrakingStateOutput { state: brakes } =
                            <BrakingStateState as grust::core::Component>::step(
                                &mut self.braking_state,
                                BrakingStateInput {
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: None,
                                    speed: self.context.speed_km_h_bis.get(),
                                    acc: self.context.acc_km_h.get(),
                                },
                            );
                        self.context.brakes.set(brakes);
                    }
                    if self.context.brakes.is_new() {
                        self.send_output(
                            O::Brakes(self.context.brakes.get(), _pedestrian_r_instant),
                            _pedestrian_r_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .pedestrian_r
                        .replace((pedestrian_r, _pedestrian_r_instant));
                    assert ! (unique . is_none () , "flow `pedestrian_r` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_delay_aeb(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                self.context.reset();
                if self.input_store.not_empty() {
                    self.reset_time_constraints(_grust_reserved_instant).await?;
                    let pedestrian_r_ref = &mut None;
                    let pedestrian_ref = &mut None;
                    let timeout_pedest_ref = &mut None;
                    let speed_km_h_ref = &mut None;
                    let timeout_timeout_pedest_ref = &mut None;
                    let pedestrian_l_ref = &mut None;
                    let _pedestrian_r_input_store = self.input_store.pedestrian_r.take();
                    *pedestrian_r_ref = _pedestrian_r_input_store.map(|(x, _)| x);
                    let _timeout_timeout_pedest_input_store =
                        self.input_store.timeout_timeout_pedest.take();
                    *timeout_timeout_pedest_ref =
                        _timeout_timeout_pedest_input_store.map(|(x, _)| x);
                    let _pedestrian_l_input_store = self.input_store.pedestrian_l.take();
                    *pedestrian_l_ref = _pedestrian_l_input_store.map(|(x, _)| x);
                    if pedestrian_l_ref.is_some() {
                        *pedestrian_ref = *pedestrian_l_ref;
                    } else {
                        if pedestrian_r_ref.is_some() {
                            *pedestrian_ref = *pedestrian_r_ref;
                        }
                    }
                    if pedestrian_ref.is_some() {
                        self.send_timer(T::TimeoutTimeoutPedest, _grust_reserved_instant)
                            .await?;
                    } else {
                        if timeout_timeout_pedest_ref.is_some() {
                            *timeout_pedest_ref = Some(());
                            if let Some((_, _timeout_timeout_pedest_instant)) =
                                _timeout_timeout_pedest_input_store
                            {
                                self.send_timer(
                                    T::TimeoutTimeoutPedest,
                                    _timeout_timeout_pedest_instant,
                                )
                                .await?;
                            }
                        }
                    }
                    let _speed_km_h_input_store = self.input_store.speed_km_h.take();
                    *speed_km_h_ref = _speed_km_h_input_store.map(|(x, _)| x);
                    if self.context.speed_km_h_bis.get() != *speed_km_h_ref {
                        self.context.speed_km_h_bis.set(*speed_km_h_ref);
                    }
                    let x = (_grust_reserved_instant
                        .duration_since(self.begin)
                        .as_millis()) as f64;
                    self.context.x.set(x);
                    if self.context.speed_km_h_bis.is_new() || self.context.x.is_new() {
                        let DeriveOutput { a_km_h: acc_km_h } =
                            <DeriveState as grust::core::Component>::step(
                                &mut self.derive,
                                DeriveInput {
                                    v_km_h: self.context.speed_km_h_bis.get(),
                                    t: x,
                                },
                            );
                        self.context.acc_km_h.set(acc_km_h);
                    }
                    if pedestrian_ref.is_some()
                        || timeout_pedest_ref.is_some()
                        || self.context.speed_km_h_bis.is_new()
                        || self.context.acc_km_h.is_new()
                    {
                        let BrakingStateOutput { state: brakes } =
                            <BrakingStateState as grust::core::Component>::step(
                                &mut self.braking_state,
                                BrakingStateInput {
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: *timeout_pedest_ref,
                                    speed: self.context.speed_km_h_bis.get(),
                                    acc: self.context.acc_km_h.get(),
                                },
                            );
                        self.context.brakes.set(brakes);
                    }
                    if self.context.brakes.is_new() {
                        self.send_output(
                            O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                            _grust_reserved_instant,
                        )
                        .await?;
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
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                self.timer
                    .send((T::DelayAeb, _grust_reserved_instant))
                    .await?;
                self.delayed = false;
                Ok(())
            }
            pub async fn handle_timeout_aeb(
                &mut self,
                _timeout_aeb_instant: std::time::Instant,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                self.reset_time_constraints(_timeout_aeb_instant).await?;
                self.context.reset();
                let x = (_timeout_aeb_instant.duration_since(self.begin).as_millis()) as f64;
                self.context.x.set(x);
                if self.context.speed_km_h_bis.is_new() || self.context.x.is_new() {
                    let DeriveOutput { a_km_h: acc_km_h } =
                        <DeriveState as grust::core::Component>::step(
                            &mut self.derive,
                            DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            },
                        );
                    self.context.acc_km_h.set(acc_km_h);
                }
                if self.context.speed_km_h_bis.is_new() || self.context.acc_km_h.is_new() {
                    let BrakingStateOutput { state: brakes } =
                        <BrakingStateState as grust::core::Component>::step(
                            &mut self.braking_state,
                            BrakingStateInput {
                                pedest: None,
                                timeout_pedest: None,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            },
                        );
                    self.context.brakes.set(brakes);
                }
                self.send_output(
                    O::Brakes(self.context.brakes.get(), _timeout_aeb_instant),
                    _timeout_aeb_instant,
                )
                .await?;
                Ok(())
            }
            #[inline]
            pub async fn reset_service_timeout(
                &mut self,
                _timeout_aeb_instant: std::time::Instant,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                self.timer
                    .send((T::TimeoutAeb, _timeout_aeb_instant))
                    .await?;
                Ok(())
            }
            #[inline]
            pub async fn reset_time_constraints(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                self.reset_service_delay(instant).await?;
                Ok(())
            }
            #[inline]
            pub async fn send_output(
                &mut self,
                output: O,
                instant: std::time::Instant,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                self.reset_service_timeout(instant).await?;
                self.output.feed(output).await?;
                Ok(())
            }
            #[inline]
            pub async fn send_timer(
                &mut self,
                timer: T,
                instant: std::time::Instant,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                self.timer.feed((timer, instant)).await?;
                Ok(())
            }
        }
    }
}
use grust::futures::{Stream, StreamExt};
pub fn run(
    INIT: std::time::Instant,
    input_stream: impl Stream<Item = runtime::RuntimeInput> + Send + 'static,
    init_signals: runtime::RuntimeInit,
) -> grust::futures::channel::mpsc::Receiver<runtime::RuntimeOutput> {
    const TIMER_CHANNEL_SIZE: usize = 3usize + 2;
    const TIMER_STREAM_SIZE: usize = 3usize + 2;
    let (timers_sink, timers_stream) = grust::futures::channel::mpsc::channel(TIMER_CHANNEL_SIZE);
    let timers_stream =
        grust::core::timer_stream::timer_stream::<_, _, TIMER_STREAM_SIZE>(timers_stream)
            .map(|(timer, deadline)| runtime::RuntimeInput::Timer(timer, deadline));
    const OUTPUT_CHANNEL_SIZE: usize = 1usize;
    let (output_sink, output_stream) = grust::futures::channel::mpsc::channel(OUTPUT_CHANNEL_SIZE);
    const PRIO_STREAM_SIZE: usize = 4usize;
    let prio_stream = grust::core::priority_stream::prio_stream::<_, _, PRIO_STREAM_SIZE>(
        grust::futures::stream::select(input_stream, timers_stream),
        runtime::RuntimeInput::order,
    );
    let service = runtime::Runtime::new(output_sink, timers_sink);
    grust::tokio::spawn(async move {
        let result = service.run_loop(INIT, prio_stream, init_signals).await;
        assert!(result.is_ok())
    });
    output_stream
}
