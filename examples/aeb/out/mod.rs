#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Braking {
    #[default]
    NoBrake,
    SoftBrake,
    UrgentBrake,
}
pub fn compute_soft_braking_distance(speed: f64) -> f64 {
    (speed * speed) / 100.0f64
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
    last_state: Braking,
}
impl grust::core::Component for BrakingStateState {
    type Input = BrakingStateInput;
    type Output = Braking;
    fn init() -> BrakingStateState {
        BrakingStateState {
            last_state: Braking::NoBrake,
        }
    }
    fn step(&mut self, input: BrakingStateInput) -> Braking {
        let state = match (input.pedest, input.timeout_pedest) {
            (Some(d), _) => {
                let state = brakes(d, input.speed);
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
    #[derive(Debug)]
    pub enum RuntimeInput {
        PedestrianR(f64, std::time::Instant),
        SpeedKmH(f64, std::time::Instant),
        PedestrianL(f64, std::time::Instant),
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
                (I::PedestrianR(this, _), I::PedestrianR(other, _)) => this.eq(other),
                (I::SpeedKmH(this, _), I::SpeedKmH(other, _)) => this.eq(other),
                (I::PedestrianL(this, _), I::PedestrianL(other, _)) => this.eq(other),
                (I::Timer(this, _), I::Timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::PedestrianR(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::SpeedKmH(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::PedestrianL(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Timer(_, _grust_reserved_instant) => *_grust_reserved_instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    #[derive(Debug, PartialEq)]
    pub enum RuntimeOutput {
        Braking(Braking, std::time::Instant),
    }
    use RuntimeOutput as O;
    #[derive(Debug, Default)]
    pub struct RuntimeInit {
        pub speed_km_h: f64,
    }
    #[derive(Debug, PartialEq)]
    pub enum RuntimeTimer {
        TimeoutTimeoutPedest,
        DelayAeb,
        TimeoutAeb,
    }
    use RuntimeTimer as T;
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
    pub struct Runtime {
        aeb: aeb_service::AebService,
        output: futures::channel::mpsc::Sender<O>,
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl Runtime {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let aeb = aeb_service::AebService::init(output.clone(), timer.clone());
            Runtime { aeb, output, timer }
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
            init_vals: RuntimeInit,
        ) -> Result<(), futures::channel::mpsc::SendError> {
            futures::pin_mut!(input);
            let mut runtime = self;
            let RuntimeInit { speed_km_h } = init_vals;
            runtime
                .aeb
                .handle_init(_grust_reserved_init_instant, speed_km_h)
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
                    I::Timer(T::DelayAeb, _grust_reserved_instant) => {
                        runtime
                            .aeb
                            .handle_delay_aeb(_grust_reserved_instant)
                            .await?;
                    }
                    I::SpeedKmH(speed_km_h, _grust_reserved_instant) => {
                        runtime
                            .aeb
                            .handle_speed_km_h(_grust_reserved_instant, speed_km_h)
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
        mod ctx_ty {
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
            pub struct Braking(super::Braking, bool);
            impl Braking {
                pub fn set(&mut self, braking: super::Braking) {
                    self.1 = self.0 != braking;
                    self.0 = braking;
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
        }
        #[derive(Clone, Copy, PartialEq, Default, Debug)]
        pub struct Context {
            pub speed_km_h: ctx_ty::SpeedKmH,
            pub braking: ctx_ty::Braking,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.speed_km_h.reset();
                self.braking.reset();
            }
        }
        #[derive(Default)]
        pub struct AebServiceStore {
            pedestrian_r: Option<(f64, std::time::Instant)>,
            timeout_timeout_pedest: Option<((), std::time::Instant)>,
            speed_km_h: Option<(f64, std::time::Instant)>,
            pedestrian_l: Option<(f64, std::time::Instant)>,
        }
        impl AebServiceStore {
            pub fn not_empty(&self) -> bool {
                self.pedestrian_r.is_some()
                    || self.timeout_timeout_pedest.is_some()
                    || self.speed_km_h.is_some()
                    || self.pedestrian_l.is_some()
            }
        }
        pub struct AebService {
            begin: std::time::Instant,
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
                let braking_state = <BrakingStateState as grust::core::Component>::init();
                AebService {
                    begin: std::time::Instant::now(),
                    context,
                    delayed,
                    input_store,
                    braking_state,
                    output,
                    timer,
                }
            }
            pub async fn handle_init(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
                speed_km_h: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_timeout(_grust_reserved_instant).await?;
                self.context.speed_km_h.set(speed_km_h);
                self.send_timer(T::TimeoutTimeoutPedest, _grust_reserved_instant)
                    .await?;
                let braking = <BrakingStateState as grust::core::Component>::step(
                    &mut self.braking_state,
                    BrakingStateInput {
                        pedest: None,
                        timeout_pedest: None,
                        speed: speed_km_h,
                    },
                );
                self.context.braking.set(braking);
                self.send_output(
                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                    _grust_reserved_instant,
                )
                .await?;
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
                    let pedestrian_ref = &mut None;
                    let pedestrian_r_ref = &mut None;
                    *pedestrian_r_ref = Some(pedestrian_r);
                    if pedestrian_r_ref.is_some() {
                        *pedestrian_ref = *pedestrian_r_ref;
                    }
                    if pedestrian_ref.is_some() {
                        self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                            .await?;
                    }
                    if pedestrian_ref.is_some() || self.context.speed_km_h.is_new() {
                        let braking = <BrakingStateState as grust::core::Component>::step(
                            &mut self.braking_state,
                            BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: None,
                                speed: self.context.speed_km_h.get(),
                            },
                        );
                        self.context.braking.set(braking);
                    }
                    if self.context.braking.is_new() {
                        self.send_output(
                            O::Braking(self.context.braking.get(), _pedestrian_r_instant),
                            _pedestrian_r_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .pedestrian_r
                        .replace((pedestrian_r, _pedestrian_r_instant));
                    assert!
                    (unique.is_none(),
                    "flow `pedestrian_r` changes twice within one minimal delay of the service, consider reducing this delay");
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
                    if timeout_pedest_ref.is_some() || self.context.speed_km_h.is_new() {
                        let braking = <BrakingStateState as grust::core::Component>::step(
                            &mut self.braking_state,
                            BrakingStateInput {
                                pedest: None,
                                timeout_pedest: *timeout_pedest_ref,
                                speed: self.context.speed_km_h.get(),
                            },
                        );
                        self.context.braking.set(braking);
                    }
                    if self.context.braking.is_new() {
                        self.send_output(
                            O::Braking(self.context.braking.get(), _timeout_timeout_pedest_instant),
                            _timeout_timeout_pedest_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .timeout_timeout_pedest
                        .replace(((), _timeout_timeout_pedest_instant));
                    assert!
                    (unique.is_none(),
                    "flow `timeout_timeout_pedest` changes twice within one minimal delay of the service, consider reducing this delay");
                }
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
                        let braking = <BrakingStateState as grust::core::Component>::step(
                            &mut self.braking_state,
                            BrakingStateInput {
                                pedest: None,
                                timeout_pedest: None,
                                speed: speed_km_h,
                            },
                        );
                        self.context.braking.set(braking);
                    }
                    if self.context.braking.is_new() {
                        self.send_output(
                            O::Braking(self.context.braking.get(), _speed_km_h_instant),
                            _speed_km_h_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .speed_km_h
                        .replace((speed_km_h, _speed_km_h_instant));
                    assert!
                    (unique.is_none(),
                    "flow `speed_km_h` changes twice within one minimal delay of the service, consider reducing this delay");
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
                        self.input_store.pedestrian_r.take(),
                        self.input_store.timeout_timeout_pedest.take(),
                        self.input_store.speed_km_h.take(),
                        self.input_store.pedestrian_l.take(),
                    ) {
                        (None, None, None, None) => {}
                        (Some((pedestrian_r, _pedestrian_r_instant)), None, None, None) => {
                            let pedestrian_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() || self.context.speed_km_h.is_new() {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: None,
                                        speed: self.context.speed_km_h.get(),
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, Some(((), _timeout_timeout_pedest_instant)), None, None) => {
                            let timeout_pedest_ref = &mut None;
                            *timeout_pedest_ref = Some(());
                            self.send_timer(
                                T::TimeoutTimeoutPedest,
                                _timeout_timeout_pedest_instant,
                            )
                            .await?;
                            if timeout_pedest_ref.is_some() || self.context.speed_km_h.is_new() {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: None,
                                        timeout_pedest: *timeout_pedest_ref,
                                        speed: self.context.speed_km_h.get(),
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((pedestrian_r, _pedestrian_r_instant)),
                            Some(((), _timeout_timeout_pedest_instant)),
                            None,
                            None,
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some()
                                || timeout_pedest_ref.is_some()
                                || self.context.speed_km_h.is_new()
                            {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: *timeout_pedest_ref,
                                        speed: self.context.speed_km_h.get(),
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, None, Some((speed_km_h, _speed_km_h_instant)), None) => {
                            self.context.speed_km_h.set(speed_km_h);
                            if self.context.speed_km_h.is_new() {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: None,
                                        timeout_pedest: None,
                                        speed: speed_km_h,
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((pedestrian_r, _pedestrian_r_instant)),
                            None,
                            Some((speed_km_h, _speed_km_h_instant)),
                            None,
                        ) => {
                            let pedestrian_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() || self.context.speed_km_h.is_new() {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: None,
                                        speed: speed_km_h,
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some(((), _timeout_timeout_pedest_instant)),
                            Some((speed_km_h, _speed_km_h_instant)),
                            None,
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            *timeout_pedest_ref = Some(());
                            self.send_timer(
                                T::TimeoutTimeoutPedest,
                                _timeout_timeout_pedest_instant,
                            )
                            .await?;
                            if timeout_pedest_ref.is_some() || self.context.speed_km_h.is_new() {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: None,
                                        timeout_pedest: *timeout_pedest_ref,
                                        speed: speed_km_h,
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((pedestrian_r, _pedestrian_r_instant)),
                            Some(((), _timeout_timeout_pedest_instant)),
                            Some((speed_km_h, _speed_km_h_instant)),
                            None,
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            self.context.speed_km_h.set(speed_km_h);
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_r_ref.is_some() {
                                *pedestrian_ref = *pedestrian_r_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some()
                                || timeout_pedest_ref.is_some()
                                || self.context.speed_km_h.is_new()
                            {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: *timeout_pedest_ref,
                                        speed: speed_km_h,
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, None, None, Some((pedestrian_l, _pedestrian_l_instant))) => {
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() || self.context.speed_km_h.is_new() {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: None,
                                        speed: self.context.speed_km_h.get(),
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((pedestrian_r, _pedestrian_r_instant)),
                            None,
                            None,
                            Some((pedestrian_l, _pedestrian_l_instant)),
                        ) => {
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            } else {
                                if pedestrian_r_ref.is_some() {
                                    *pedestrian_ref = *pedestrian_r_ref;
                                }
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() || self.context.speed_km_h.is_new() {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: None,
                                        speed: self.context.speed_km_h.get(),
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some(((), _timeout_timeout_pedest_instant)),
                            None,
                            Some((pedestrian_l, _pedestrian_l_instant)),
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
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
                            if pedestrian_ref.is_some()
                                || timeout_pedest_ref.is_some()
                                || self.context.speed_km_h.is_new()
                            {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: *timeout_pedest_ref,
                                        speed: self.context.speed_km_h.get(),
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((pedestrian_r, _pedestrian_r_instant)),
                            Some(((), _timeout_timeout_pedest_instant)),
                            None,
                            Some((pedestrian_l, _pedestrian_l_instant)),
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            } else {
                                if pedestrian_r_ref.is_some() {
                                    *pedestrian_ref = *pedestrian_r_ref;
                                }
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some()
                                || timeout_pedest_ref.is_some()
                                || self.context.speed_km_h.is_new()
                            {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: *timeout_pedest_ref,
                                        speed: self.context.speed_km_h.get(),
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((speed_km_h, _speed_km_h_instant)),
                            Some((pedestrian_l, _pedestrian_l_instant)),
                        ) => {
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                                    .await?;
                            }
                            self.context.speed_km_h.set(speed_km_h);
                            if pedestrian_ref.is_some() || self.context.speed_km_h.is_new() {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: None,
                                        speed: speed_km_h,
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((pedestrian_r, _pedestrian_r_instant)),
                            None,
                            Some((speed_km_h, _speed_km_h_instant)),
                            Some((pedestrian_l, _pedestrian_l_instant)),
                        ) => {
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            self.context.speed_km_h.set(speed_km_h);
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            } else {
                                if pedestrian_r_ref.is_some() {
                                    *pedestrian_ref = *pedestrian_r_ref;
                                }
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some() || self.context.speed_km_h.is_new() {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: None,
                                        speed: speed_km_h,
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some(((), _timeout_timeout_pedest_instant)),
                            Some((speed_km_h, _speed_km_h_instant)),
                            Some((pedestrian_l, _pedestrian_l_instant)),
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            }
                            self.context.speed_km_h.set(speed_km_h);
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
                            if pedestrian_ref.is_some()
                                || timeout_pedest_ref.is_some()
                                || self.context.speed_km_h.is_new()
                            {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: *timeout_pedest_ref,
                                        speed: speed_km_h,
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((pedestrian_r, _pedestrian_r_instant)),
                            Some(((), _timeout_timeout_pedest_instant)),
                            Some((speed_km_h, _speed_km_h_instant)),
                            Some((pedestrian_l, _pedestrian_l_instant)),
                        ) => {
                            let timeout_pedest_ref = &mut None;
                            let pedestrian_l_ref = &mut None;
                            let pedestrian_ref = &mut None;
                            let pedestrian_r_ref = &mut None;
                            *pedestrian_l_ref = Some(pedestrian_l);
                            self.context.speed_km_h.set(speed_km_h);
                            *pedestrian_r_ref = Some(pedestrian_r);
                            if pedestrian_l_ref.is_some() {
                                *pedestrian_ref = *pedestrian_l_ref;
                            } else {
                                if pedestrian_r_ref.is_some() {
                                    *pedestrian_ref = *pedestrian_r_ref;
                                }
                            }
                            if pedestrian_ref.is_some() {
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            } else {
                                *timeout_pedest_ref = Some(());
                                self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_r_instant)
                                    .await?;
                            }
                            if pedestrian_ref.is_some()
                                || timeout_pedest_ref.is_some()
                                || self.context.speed_km_h.is_new()
                            {
                                let braking = <BrakingStateState as grust::core::Component>::step(
                                    &mut self.braking_state,
                                    BrakingStateInput {
                                        pedest: *pedestrian_ref,
                                        timeout_pedest: *timeout_pedest_ref,
                                        speed: speed_km_h,
                                    },
                                );
                                self.context.braking.set(braking);
                            }
                            if self.context.braking.is_new() {
                                self.send_output(
                                    O::Braking(self.context.braking.get(), _grust_reserved_instant),
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
                self.delayed = false;
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
                    let pedestrian_l_ref = &mut None;
                    let pedestrian_ref = &mut None;
                    *pedestrian_l_ref = Some(pedestrian_l);
                    if pedestrian_l_ref.is_some() {
                        *pedestrian_ref = *pedestrian_l_ref;
                    }
                    if pedestrian_ref.is_some() {
                        self.send_timer(T::TimeoutTimeoutPedest, _pedestrian_l_instant)
                            .await?;
                    }
                    if pedestrian_ref.is_some() || self.context.speed_km_h.is_new() {
                        let braking = <BrakingStateState as grust::core::Component>::step(
                            &mut self.braking_state,
                            BrakingStateInput {
                                pedest: *pedestrian_ref,
                                timeout_pedest: None,
                                speed: self.context.speed_km_h.get(),
                            },
                        );
                        self.context.braking.set(braking);
                    }
                    if self.context.braking.is_new() {
                        self.send_output(
                            O::Braking(self.context.braking.get(), _pedestrian_l_instant),
                            _pedestrian_l_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .pedestrian_l
                        .replace((pedestrian_l, _pedestrian_l_instant));
                    assert!
                    (unique.is_none(),
                    "flow `pedestrian_l` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_timeout_aeb(
                &mut self,
                _timeout_aeb_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constraints(_timeout_aeb_instant).await?;
                self.context.reset();
                self.send_output(
                    O::Braking(self.context.braking.get(), _timeout_aeb_instant),
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
            #[inline]
            pub async fn reset_time_constraints(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_delay(instant).await?;
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
use futures::{Stream, StreamExt};
pub fn run(
    INIT: std::time::Instant,
    input_stream: impl Stream<Item = runtime::RuntimeInput> + Send + 'static,
    init_signals: runtime::RuntimeInit,
) -> futures::channel::mpsc::Receiver<runtime::RuntimeOutput> {
    const TIMER_CHANNEL_SIZE: usize = 3usize + 2;
    let (timers_sink, timers_stream) = futures::channel::mpsc::channel(TIMER_CHANNEL_SIZE);
    let timers_stream = timers_stream.map(
        |(timer, instant): (runtime::RuntimeTimer, std::time::Instant)| {
            let deadline = instant + timer_stream::Timing::get_duration(&timer);
            runtime::RuntimeInput::Timer(timer, deadline)
        },
    );
    const OUTPUT_CHANNEL_SIZE: usize = 1usize;
    let (output_sink, output_stream) = futures::channel::mpsc::channel(OUTPUT_CHANNEL_SIZE);
    const PRIO_STREAM_SIZE: usize = 100usize;
    let prio_stream = priority_stream::prio_stream::<_, _, PRIO_STREAM_SIZE>(
        futures::stream::select(input_stream, timers_stream),
        runtime::RuntimeInput::order,
    );
    let service = runtime::Runtime::new(output_sink, timers_sink);
    tokio::spawn(async move {
        let result = service.run_loop(INIT, prio_stream, init_signals).await;
        assert!(result.is_ok())
    });
    output_stream
}
