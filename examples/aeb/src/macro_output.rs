#[derive(Clone, Copy, PartialEq, Default)]
pub enum Braking {
    #[default]
    UrgentBrake,
    SoftBrake,
    NoBrake,
}
pub fn compute_soft_braking_distance(speed: f64) -> f64 {
    (speed * speed) / 100.0
}
pub fn brakes(distance: f64, speed: f64) -> Braking {
    let braking_distance = compute_soft_braking_distance(speed);
    let response = if braking_distance < distance {
        Braking::SoftBrake
    } else {
        Braking::UrgentBrake
    };
    response
}
pub struct BrakingStateInput {
    pub pedest: Option<f64>,
    pub timeout_pedest: Option<()>,
    pub speed: f64,
}
pub struct BrakingStateState {
    mem: Braking,
}
impl BrakingStateState {
    pub fn init() -> BrakingStateState {
        BrakingStateState {
            mem: Default::default(),
        }
    }
    pub fn step(&mut self, input: BrakingStateInput) -> Braking {
        let state = match (input.pedest, input.timeout_pedest) {
            (Some(d), _) => {
                let state = brakes(d, input.speed);
                state
            }
            (_, Some(_)) => {
                let state = Braking::NoBrake;
                state
            }
            (_, _) => self.mem,
        };
        self.mem = state;
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
        PedestrianL(f64, std::time::Instant),
        PedestrianR(f64, std::time::Instant),
        SpeedKmH(f64, std::time::Instant),
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
                (I::PedestrianL(this, _), I::PedestrianL(other, _)) => this.eq(other),
                (I::PedestrianR(this, _), I::PedestrianR(other, _)) => this.eq(other),
                (I::SpeedKmH(this, _), I::SpeedKmH(other, _)) => this.eq(other),
                (I::Timer(this, _), I::Timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::PedestrianL(_, instant) => *instant,
                I::PedestrianR(_, instant) => *instant,
                I::SpeedKmH(_, instant) => *instant,
                I::Timer(_, instant) => *instant,
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
            init_instant: std::time::Instant,
            input: impl futures::Stream<Item = I>,
        ) -> Result<(), futures::channel::mpsc::SendError> {
            futures::pin_mut!(input);
            let mut runtime = self;
            runtime.send_timer(T::TimeoutAeb, init_instant).await?;
            runtime
                .send_timer(T::TimeoutTimeoutPedest, init_instant)
                .await?;
            while let Some(input) = input.next().await {
                match input {
                    I::PedestrianL(pedestrian_l, instant) => {
                        runtime
                            .aeb
                            .handle_pedestrian_l(instant, pedestrian_l)
                            .await?;
                    }
                    I::PedestrianR(pedestrian_r, instant) => {
                        runtime
                            .aeb
                            .handle_pedestrian_r(instant, pedestrian_r)
                            .await?;
                    }
                    I::Timer(T::DelayAeb, instant) => {
                        runtime.aeb.handle_delay_aeb(instant).await?;
                    }
                    I::Timer(T::TimeoutAeb, instant) => {
                        runtime.aeb.handle_timeout_aeb(instant).await?;
                    }
                    I::SpeedKmH(speed_km_h, instant) => {
                        runtime.aeb.handle_speed_km_h(instant, speed_km_h).await?;
                    }
                    I::Timer(T::TimeoutTimeoutPedest, instant) => {
                        runtime.aeb.handle_timeout_timeout_pedest(instant).await?;
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
                self.0 = brakes;
                self.1 = true;
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
        pub struct Context {
            pub brakes: Brakes,
            pub speed_km_h: SpeedKmH,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.brakes.reset();
                self.speed_km_h.reset();
            }
        }
        #[derive(Default)]
        pub struct AebServiceStore {
            pedestrian_l: Option<(f64, std::time::Instant)>,
            timeout_timeout_pedest: Option<((), std::time::Instant)>,
            pedestrian_r: Option<(f64, std::time::Instant)>,
            speed_km_h: Option<(f64, std::time::Instant)>,
        }
        impl AebServiceStore {
            pub fn not_empty(&self) -> bool {
                self.pedestrian_l.is_some()
                    || self.timeout_timeout_pedest.is_some()
                    || self.pedestrian_r.is_some()
                    || self.speed_km_h.is_some()
            }
        }
        pub struct AebService {
            context: Context,
            delayed: bool,
            input_store: AebServiceStore,
            braking_state: BrakingStateState,
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
                AebService {
                    context,
                    delayed,
                    input_store,
                    braking_state,
                    output,
                    timer,
                }
            }
            pub async fn handle_pedestrian_l(
                &mut self,
                pedestrian_l_instant: std::time::Instant,
                pedestrian_l: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(pedestrian_l_instant).await?;
                    self.context.reset();
                    let pedestrian_ref = &mut None;
                    let pedestrian_l_ref = &mut None;
                    *pedestrian_l_ref = Some(pedestrian_l);
                    if pedestrian_l_ref.is_some() {
                        *pedestrian_ref = *pedestrian_l_ref;
                    }
                    if pedestrian_ref.is_some() {
                        self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                            .await?;
                    }
                    if pedestrian_ref.is_some() {
                        let brakes = self.braking_state.step(BrakingStateInput {
                            speed: self.context.speed_km_h.get(),
                            pedest: *pedestrian_ref,
                            timeout_pedest: None,
                        });
                        self.context.brakes.set(brakes);
                    }
                    self.send_output(O::Brakes(self.context.brakes.get(), pedestrian_l_instant))
                        .await?;
                } else {
                    let unique = self
                        .input_store
                        .pedestrian_l
                        .replace((pedestrian_l, pedestrian_l_instant));
                    assert!(unique.is_none(), "pedestrian_l changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_timeout_timeout_pedest(
                &mut self,
                timeout_timeout_pedest_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(timeout_timeout_pedest_instant)
                        .await?;
                    self.context.reset();
                    let timeout_pedest_ref = &mut None;
                    *timeout_pedest_ref = Some(());
                    self.send_timer(T::TimeoutTimeoutPedest, timeout_timeout_pedest_instant)
                        .await?;
                    if timeout_pedest_ref.is_some() {
                        let brakes = self.braking_state.step(BrakingStateInput {
                            speed: self.context.speed_km_h.get(),
                            pedest: None,
                            timeout_pedest: *timeout_pedest_ref,
                        });
                        self.context.brakes.set(brakes);
                    }
                    self.send_output(O::Brakes(
                        self.context.brakes.get(),
                        timeout_timeout_pedest_instant,
                    ))
                    .await?;
                } else {
                    let unique = self
                        .input_store
                        .timeout_timeout_pedest
                        .replace(((), timeout_timeout_pedest_instant));
                    assert!(
                        unique.is_none(),
                        "timeout_timeout_pedest changes too frequently"
                    );
                }
                Ok(())
            }
            pub async fn handle_pedestrian_r(
                &mut self,
                pedestrian_r_instant: std::time::Instant,
                pedestrian_r: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(pedestrian_r_instant).await?;
                    self.context.reset();
                    let pedestrian_r_ref = &mut None;
                    let pedestrian_ref = &mut None;
                    *pedestrian_r_ref = Some(pedestrian_r);
                    if pedestrian_r_ref.is_some() {
                        *pedestrian_ref = *pedestrian_r_ref;
                    }
                    if pedestrian_ref.is_some() {
                        self.send_timer(T::TimeoutTimeoutPedest, pedestrian_r_instant)
                            .await?;
                    }
                    if pedestrian_ref.is_some() {
                        let brakes = self.braking_state.step(BrakingStateInput {
                            speed: self.context.speed_km_h.get(),
                            pedest: *pedestrian_ref,
                            timeout_pedest: None,
                        });
                        self.context.brakes.set(brakes);
                    }
                    self.send_output(O::Brakes(self.context.brakes.get(), pedestrian_r_instant))
                        .await?;
                } else {
                    let unique = self
                        .input_store
                        .pedestrian_r
                        .replace((pedestrian_r, pedestrian_r_instant));
                    assert!(unique.is_none(), "pedestrian_r changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_delay_aeb(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.context.reset();
                if self.input_store.not_empty() {
                    self.reset_time_constrains(instant).await?;
                    match (
                        self.input_store.pedestrian_l.take(),
                        self.input_store.timeout_timeout_pedest.take(),
                        self.input_store.pedestrian_r.take(),
                        self.input_store.speed_km_h.take(),
                    ) {
                        (None, None, None, None) => {}
                        (Some((pedestrian_l, pedestrian_l_instant)), None, None, None) => {
                            let pedestrian_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: None,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (None, Some(((), timeout_timeout_pedest_instant)), None, None) => {
                            let timeout_pedest_ref = &mut None;
                            *timeout_pedest_ref = Some(());
                            self.send_timer(
                                T::TimeoutTimeoutPedest,
                                timeout_timeout_pedest_instant,
                            )
                            .await?;
                            if timeout_pedest_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: None,
                                    timeout_pedest: *timeout_pedest_ref,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (
                            Some((pedestrian_l, pedestrian_l_instant)),
                            Some(((), timeout_timeout_pedest_instant)),
                            None,
                            None,
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() || timeout_pedest_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: *timeout_pedest_ref,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (None, None, Some((pedestrian_r, pedestrian_r_instant)), None) => {
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_r_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: None,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (
                            Some((pedestrian_l, pedestrian_l_instant)),
                            None,
                            Some((pedestrian_r, pedestrian_r_instant)),
                            None,
                        ) => {
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
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
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: None,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (
                            None,
                            Some(((), timeout_timeout_pedest_instant)),
                            Some((pedestrian_r, pedestrian_r_instant)),
                            None,
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(
                                    T::TimeoutTimeoutPedest,
                                    timeout_timeout_pedest_instant,
                                )
                                .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(
                                    T::TimeoutTimeoutPedest,
                                    timeout_timeout_pedest_instant,
                                )
                                .await?;
                            }
                            if pedestrian_ref.is_some() || timeout_pedest_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: *timeout_pedest_ref,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (
                            Some((pedestrian_l, pedestrian_l_instant)),
                            Some(((), timeout_timeout_pedest_instant)),
                            Some((pedestrian_r, pedestrian_r_instant)),
                            None,
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
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
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() || timeout_pedest_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: *timeout_pedest_ref,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (None, None, None, Some((speed_km_h, speed_km_h_instant))) => {
                            self.context.speed_km_h.set(speed_km_h);
                        }
                        (
                            Some((pedestrian_l, pedestrian_l_instant)),
                            None,
                            None,
                            Some((speed_km_h, speed_km_h_instant)),
                        ) => {
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: None,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (
                            None,
                            Some(((), timeout_timeout_pedest_instant)),
                            None,
                            Some((speed_km_h, speed_km_h_instant)),
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            *timeout_pedest_ref = Some(());
                            self.send_timer(
                                T::TimeoutTimeoutPedest,
                                timeout_timeout_pedest_instant,
                            )
                            .await?;
                            if timeout_pedest_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: None,
                                    timeout_pedest: *timeout_pedest_ref,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (
                            Some((pedestrian_l, pedestrian_l_instant)),
                            Some(((), timeout_timeout_pedest_instant)),
                            None,
                            Some((speed_km_h, speed_km_h_instant)),
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() || timeout_pedest_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: *timeout_pedest_ref,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (
                            None,
                            None,
                            Some((pedestrian_r, pedestrian_r_instant)),
                            Some((speed_km_h, speed_km_h_instant)),
                        ) => {
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_r_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: None,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (
                            Some((pedestrian_l, pedestrian_l_instant)),
                            None,
                            Some((pedestrian_r, pedestrian_r_instant)),
                            Some((speed_km_h, speed_km_h_instant)),
                        ) => {
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
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
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: None,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (
                            None,
                            Some(((), timeout_timeout_pedest_instant)),
                            Some((pedestrian_r, pedestrian_r_instant)),
                            Some((speed_km_h, speed_km_h_instant)),
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(
                                    T::TimeoutTimeoutPedest,
                                    timeout_timeout_pedest_instant,
                                )
                                .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(
                                    T::TimeoutTimeoutPedest,
                                    timeout_timeout_pedest_instant,
                                )
                                .await?;
                            }
                            if pedestrian_ref.is_some() || timeout_pedest_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: *timeout_pedest_ref,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
                        }
                        (
                            Some((pedestrian_l, pedestrian_l_instant)),
                            Some(((), timeout_timeout_pedest_instant)),
                            Some((pedestrian_r, pedestrian_r_instant)),
                            Some((speed_km_h, speed_km_h_instant)),
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
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
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, pedestrian_l_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() || timeout_pedest_ref.is_some() {
                                let brakes = self.braking_state.step(BrakingStateInput {
                                    speed: self.context.speed_km_h.get(),
                                    pedest: *pedestrian_ref,
                                    timeout_pedest: *timeout_pedest_ref,
                                });
                                self.context.brakes.set(brakes);
                            }
                            self.send_output(O::Brakes(self.context.brakes.get(), instant))
                                .await?;
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
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer.send((T::DelayAeb, instant)).await?;
                Ok(())
            }
            pub async fn handle_timeout_aeb(
                &mut self,
                timeout_aeb_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constrains(timeout_aeb_instant).await?;
                self.context.reset();
                self.send_output(O::Brakes(self.context.brakes.get(), timeout_aeb_instant))
                    .await?;
                Ok(())
            }
            #[inline]
            pub async fn reset_service_timeout(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer.send((T::TimeoutAeb, instant)).await?;
                Ok(())
            }
            pub async fn handle_speed_km_h(
                &mut self,
                speed_km_h_instant: std::time::Instant,
                speed_km_h: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(speed_km_h_instant).await?;
                    self.context.reset();
                    self.context.speed_km_h.set(speed_km_h);
                } else {
                    let unique = self
                        .input_store
                        .speed_km_h
                        .replace((speed_km_h, speed_km_h_instant));
                    assert!(unique.is_none(), "speed_km_h changes too frequently");
                }
                Ok(())
            }
            #[inline]
            pub async fn reset_time_constrains(
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
