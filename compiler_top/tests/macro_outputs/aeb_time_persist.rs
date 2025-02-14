#[derive(Clone, Copy, PartialEq, Default)]
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
pub struct DeriveState {
    last_a: f64,
    last_t: f64,
    last_v: f64,
    last_x: bool,
}
impl DeriveState {
    pub fn init() -> DeriveState {
        DeriveState {
            last_a: 0.0f64,
            last_t: 0.0f64,
            last_v: 0.0f64,
            last_x: false,
        }
    }
    pub fn step(&mut self, input: DeriveInput) -> f64 {
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
        a_km_h
    }
}
pub struct BrakingStateInput {
    pub pedest: Option<f64>,
    pub timeout_pedest: Option<()>,
    pub speed: f64,
    pub acc: f64,
}
pub struct BrakingStateState {
    last_state: Braking,
}
impl BrakingStateState {
    pub fn init() -> BrakingStateState {
        BrakingStateState {
            last_state: Braking::NoBrake,
        }
    }
    pub fn step(&mut self, input: BrakingStateInput) -> Braking {
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
        state
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
        TimeoutTimeoutPedest,
        DelayAeb,
        TimeoutAeb,
    }
    impl timer_stream::Timing for RuntimeTimer {
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
    pub enum RuntimeInput {
        SpeedKmH(f64, std::time::Instant),
        PedestrianL(f64, std::time::Instant),
        PedestrianR(f64, std::time::Instant),
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
    pub enum RuntimeOutput {
        Brakes(Braking, std::time::Instant),
    }
    pub struct Runtime {
        aeb: aeb_service::AebService,
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl Runtime {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let aeb = aeb_service::AebService::init(output, timer.clone());
            Runtime { aeb, timer }
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
                .send_timer(T::TimeoutAeb, _grust_reserved_init_instant)
                .await?;
            runtime
                .send_timer(T::TimeoutTimeoutPedest, _grust_reserved_init_instant)
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
        use futures::{sink::SinkExt, stream::StreamExt};
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Brakes(Braking, bool);
        impl Brakes {
            fn set(&mut self, brakes: Braking) {
                self.1 = self.0 != brakes;
                self.0 = brakes;
            }
            fn get(&self) -> Braking {
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
        pub struct SpeedKmHBis(f64, bool);
        impl SpeedKmHBis {
            fn set(&mut self, speed_km_h_bis: f64) {
                self.1 = self.0 != speed_km_h_bis;
                self.0 = speed_km_h_bis;
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
                self.1 = self.0 != x;
                self.0 = x;
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
        pub struct AccKmH(f64, bool);
        impl AccKmH {
            fn set(&mut self, acc_km_h: f64) {
                self.1 = self.0 != acc_km_h;
                self.0 = acc_km_h;
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
            pub brakes: Brakes,
            pub speed_km_h_bis: SpeedKmHBis,
            pub x: X,
            pub acc_km_h: AccKmH,
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
            braking_state: BrakingStateState,
            derive: DeriveState,
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        }
        impl AebService {
            pub fn init(
                output: futures::channel::mpsc::Sender<O>,
                timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
            ) -> AebService {
                let context = Context::init();
                let delayed = true;
                let input_store = Default::default();
                let braking_state = BrakingStateState::init();
                let derive = DeriveState::init();
                AebService {
                    begin: std::time::Instant::now(),
                    context,
                    delayed,
                    input_store,
                    braking_state,
                    derive,
                    output,
                    timer,
                }
            }
            pub async fn handle_timeout_aeb(
                &mut self,
                _timeout_aeb_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constraints(_timeout_aeb_instant).await?;
                self.context.reset();
                let acc_km_h = self.derive.step(DeriveInput {
                    v_km_h: self.context.speed_km_h_bis.get(),
                    t: self.context.x.get(),
                });
                self.context.acc_km_h.set(acc_km_h);
                let brakes = self.braking_state.step(BrakingStateInput {
                    pedest: None,
                    timeout_pedest: None,
                    speed: self.context.speed_km_h_bis.get(),
                    acc: self.context.acc_km_h.get(),
                });
                self.context.brakes.set(brakes);
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
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer
                    .send((T::TimeoutAeb, _timeout_aeb_instant))
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
                    let speed_km_h_ref = &mut None;
                    *speed_km_h_ref = Some(speed_km_h);
                    if self.context.speed_km_h_bis.get() != *speed_km_h_ref {
                        self.context.speed_km_h_bis.set(*speed_km_h_ref);
                    }
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
            pub async fn handle_pedestrian_l(
                &mut self,
                _pedestrian_l_instant: std::time::Instant,
                pedestrian_l: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
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
                    let acc_km_h = self.derive.step(DeriveInput {
                        v_km_h: self.context.speed_km_h_bis.get(),
                        t: x,
                    });
                    self.context.acc_km_h.set(acc_km_h);
                    let brakes = self.braking_state.step(BrakingStateInput {
                        pedest: *pedestrian_ref,
                        timeout_pedest: None,
                        speed: self.context.speed_km_h_bis.get(),
                        acc: self.context.acc_km_h.get(),
                    });
                    self.context.brakes.set(brakes);
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
                    assert!(
                        unique.is_none(),
                        "flow `pedestrian_l` changes too frequently"
                    );
                }
                Ok(())
            }
            pub async fn handle_timeout_timeout_pedest(
                &mut self,
                _timeout_timeout_pedest_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
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
                    let acc_km_h = self.derive.step(DeriveInput {
                        v_km_h: self.context.speed_km_h_bis.get(),
                        t: x,
                    });
                    self.context.acc_km_h.set(acc_km_h);
                    let brakes = self.braking_state.step(BrakingStateInput {
                        pedest: None,
                        timeout_pedest: *timeout_pedest_ref,
                        speed: self.context.speed_km_h_bis.get(),
                        acc: self.context.acc_km_h.get(),
                    });
                    self.context.brakes.set(brakes);
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
                    assert!(
                        unique.is_none(),
                        "flow `timeout_timeout_pedest` changes too frequently"
                    );
                }
                Ok(())
            }
            pub async fn handle_pedestrian_r(
                &mut self,
                _pedestrian_r_instant: std::time::Instant,
                pedestrian_r: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
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
                    let acc_km_h = self.derive.step(DeriveInput {
                        v_km_h: self.context.speed_km_h_bis.get(),
                        t: x,
                    });
                    self.context.acc_km_h.set(acc_km_h);
                    let brakes = self.braking_state.step(BrakingStateInput {
                        pedest: *pedestrian_ref,
                        timeout_pedest: None,
                        speed: self.context.speed_km_h_bis.get(),
                        acc: self.context.acc_km_h.get(),
                    });
                    self.context.brakes.set(brakes);
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
                    assert!(
                        unique.is_none(),
                        "flow `pedestrian_r` changes too frequently"
                    );
                }
                Ok(())
            }
            pub async fn handle_delay_aeb(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.context.reset();
                if self.input_store.not_empty() {
                    self.reset_time_constraints(_grust_reserved_instant).await?;
                    match (
                        self.input_store.speed_km_h.take(),
                        self.input_store.pedestrian_l.take(),
                        self.input_store.timeout_timeout_pedest.take(),
                        self.input_store.pedestrian_r.take(),
                    ) {
                        (None, None, None, None) => {}
                        (Some((speed_km_h, _speed_km_h_instant)), None, None, None) => {
                            let speed_km_h_ref = &mut None;
                            *speed_km_h_ref = Some(speed_km_h);
                            if self.context.speed_km_h_bis.get() != *speed_km_h_ref {
                                self.context.speed_km_h_bis.set(*speed_km_h_ref);
                            }
                            let x =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x.set(x);
                        }
                        (None, Some((pedestrian_l, _pedestrian_l_instant)), None, None) => {
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
                            let x = (_pedestrian_l_instant.duration_since(self.begin).as_millis())
                                as f64;
                            self.context.x.set(x);
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: None,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((speed_km_h, _speed_km_h_instant)),
                            Some((pedestrian_l, _pedestrian_l_instant)),
                            None,
                            None,
                        ) => {
                            let pedestrian_ref = &mut None;
                            let speed_km_h_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            }
                            *speed_km_h_ref = Some(speed_km_h);
                            let x =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.speed_km_h_bis.get() != *speed_km_h_ref {
                                self.context.speed_km_h_bis.set(*speed_km_h_ref);
                            }
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: None,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, None, Some(((), _timeout_timeout_pedest_instant)), None) => {
                            let timeout_pedest_ref = &mut None;
                            *timeout_pedest_ref = Some(());
                            self.send_timer(
                                T::TimeoutTimeoutPedest,
                                _timeout_timeout_pedest_instant,
                            )
                            .await?;
                            let x = (_timeout_timeout_pedest_instant
                                .duration_since(self.begin)
                                .as_millis()) as f64;
                            self.context.x.set(x);
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: None,
                                timeout_pedest: *timeout_pedest_ref,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((speed_km_h, _speed_km_h_instant)),
                            None,
                            Some(((), _timeout_timeout_pedest_instant)),
                            None,
                        ) => {
                            let speed_km_h_ref = &mut None;
                            let timeout_pedest_ref = &mut None;
                            *timeout_pedest_ref = Some(());
                            self.send_timer(
                                T::TimeoutTimeoutPedest,
                                _timeout_timeout_pedest_instant,
                            )
                            .await?;
                            *speed_km_h_ref = Some(speed_km_h);
                            let x =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.speed_km_h_bis.get() != *speed_km_h_ref {
                                self.context.speed_km_h_bis.set(*speed_km_h_ref);
                            }
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: None,
                                timeout_pedest: *timeout_pedest_ref,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((pedestrian_l, _pedestrian_l_instant)),
                            Some(((), _timeout_timeout_pedest_instant)),
                            None,
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            let x = (_pedestrian_l_instant.duration_since(self.begin).as_millis())
                                as f64;
                            self.context.x.set(x);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            }
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: *timeout_pedest_ref,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((speed_km_h, _speed_km_h_instant)),
                            Some((pedestrian_l, _pedestrian_l_instant)),
                            Some(((), _timeout_timeout_pedest_instant)),
                            None,
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let speed_km_h_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            }
                            *speed_km_h_ref = Some(speed_km_h);
                            let x =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.speed_km_h_bis.get() != *speed_km_h_ref {
                                self.context.speed_km_h_bis.set(*speed_km_h_ref);
                            }
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: *timeout_pedest_ref,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, None, None, Some((pedestrian_r, _pedestrian_r_instant))) => {
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
                            let x = (_pedestrian_r_instant.duration_since(self.begin).as_millis())
                                as f64;
                            self.context.x.set(x);
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: None,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((speed_km_h, _speed_km_h_instant)),
                            None,
                            None,
                            Some((pedestrian_r, _pedestrian_r_instant)),
                        ) => {
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let speed_km_h_ref = &mut None;
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            }
                            *speed_km_h_ref = Some(speed_km_h);
                            let x =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.speed_km_h_bis.get() != *speed_km_h_ref {
                                self.context.speed_km_h_bis.set(*speed_km_h_ref);
                            }
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: None,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((pedestrian_l, _pedestrian_l_instant)),
                            None,
                            Some((pedestrian_r, _pedestrian_r_instant)),
                        ) => {
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            *pedestrian_r_ref = Some(pedestrian_r);
                            *pedestrian_l_ref = Some(pedestrian_l);
                            let x = (_pedestrian_l_instant.duration_since(self.begin).as_millis())
                                as f64;
                            self.context.x.set(x);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            } else {
                                if pedestrian_l_ref.is_some() {
                                    *pedestrian_ref = *pedestrian_l_ref;
                                }
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            }
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: None,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((speed_km_h, _speed_km_h_instant)),
                            Some((pedestrian_l, _pedestrian_l_instant)),
                            None,
                            Some((pedestrian_r, _pedestrian_r_instant)),
                        ) => {
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let speed_km_h_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            *pedestrian_r_ref = Some(pedestrian_r);
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            } else {
                                if pedestrian_l_ref.is_some() {
                                    *pedestrian_ref = *pedestrian_l_ref;
                                }
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            }
                            *speed_km_h_ref = Some(speed_km_h);
                            let x =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.speed_km_h_bis.get() != *speed_km_h_ref {
                                self.context.speed_km_h_bis.set(*speed_km_h_ref);
                            }
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: None,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some(((), _timeout_timeout_pedest_instant)),
                            Some((pedestrian_r, _pedestrian_r_instant)),
                        ) => {
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let timeout_pedest_ref = &mut None;
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            }
                            let x = (_timeout_timeout_pedest_instant
                                .duration_since(self.begin)
                                .as_millis()) as f64;
                            self.context.x.set(x);
                            if pedestrian_ref.is_some() {
                                self.send_timer(
                                    T::TimeoutTimeoutPedest,
                                    _timeout_timeout_pedest_instant,
                                )
                                .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(
                                    T::TimeoutTimeoutPedest,
                                    _timeout_timeout_pedest_instant,
                                )
                                .await?;
                            }
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: *timeout_pedest_ref,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((speed_km_h, _speed_km_h_instant)),
                            None,
                            Some(((), _timeout_timeout_pedest_instant)),
                            Some((pedestrian_r, _pedestrian_r_instant)),
                        ) => {
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let speed_km_h_ref = &mut None;
                            let timeout_pedest_ref = &mut None;
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(
                                    T::TimeoutTimeoutPedest,
                                    _timeout_timeout_pedest_instant,
                                )
                                .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(
                                    T::TimeoutTimeoutPedest,
                                    _timeout_timeout_pedest_instant,
                                )
                                .await?;
                            }
                            *speed_km_h_ref = Some(speed_km_h);
                            let x =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.speed_km_h_bis.get() != *speed_km_h_ref {
                                self.context.speed_km_h_bis.set(*speed_km_h_ref);
                            }
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: *timeout_pedest_ref,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((pedestrian_l, _pedestrian_l_instant)),
                            Some(((), _timeout_timeout_pedest_instant)),
                            Some((pedestrian_r, _pedestrian_r_instant)),
                        ) => {
                            let pedestrian_r_ref = &mut None;
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            *pedestrian_r_ref = Some(pedestrian_r);
                            *pedestrian_l_ref = Some(pedestrian_l);
                            let x = (_pedestrian_l_instant.duration_since(self.begin).as_millis())
                                as f64;
                            self.context.x.set(x);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            } else {
                                if pedestrian_l_ref.is_some() {
                                    *pedestrian_ref = *pedestrian_l_ref;
                                }
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            }
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: *timeout_pedest_ref,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((speed_km_h, _speed_km_h_instant)),
                            Some((pedestrian_l, _pedestrian_l_instant)),
                            Some(((), _timeout_timeout_pedest_instant)),
                            Some((pedestrian_r, _pedestrian_r_instant)),
                        ) => {
                            let pedestrian_r_ref = &mut None;
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let speed_km_h_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            *pedestrian_r_ref = Some(pedestrian_r);
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            } else {
                                if pedestrian_l_ref.is_some() {
                                    *pedestrian_ref = *pedestrian_l_ref;
                                }
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            }
                            *speed_km_h_ref = Some(speed_km_h);
                            let x =
                                (_speed_km_h_instant.duration_since(self.begin).as_millis()) as f64;
                            self.context.x.set(x);
                            if self.context.speed_km_h_bis.get() != *speed_km_h_ref {
                                self.context.speed_km_h_bis.set(*speed_km_h_ref);
                            }
                            let acc_km_h = self.derive.step(DeriveInput {
                                v_km_h: self.context.speed_km_h_bis.get(),
                                t: x,
                            });
                            self.context.acc_km_h.set(acc_km_h);
                            let brakes = self.braking_state.step(BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: *timeout_pedest_ref,
                                speed: self.context.speed_km_h_bis.get(),
                                acc: self.context.acc_km_h.get(),
                            });
                            self.context.brakes.set(brakes);
                            if self.context.brakes.is_new() {
                                self.send_output(
                                    O::Brakes(self.context.brakes.get(), _grust_reserved_instant),
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
                    .send((T::DelayAeb, _grust_reserved_instant))
                    .await?;
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
