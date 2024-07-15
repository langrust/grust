#[derive(Clone, Copy, PartialEq, Default)]
pub enum Braking {
    #[default]
    NoBrake,
    SoftBrake,
    UrgentBrake,
}
pub fn compute_soft_braking_distance(speed: f64) -> f64 {
    speed * speed / 100.0
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
    pub pedest: Option<Result<f64, ()>>,
    pub speed: f64,
}
pub struct BrakingStateState {
    mem: Braking,
}
impl BrakingStateState {
    pub fn init() -> BrakingStateState {
        BrakingStateState {
            mem: Braking::NoBrake,
        }
    }
    pub fn step(&mut self, input: BrakingStateInput) -> Braking {
        let previous_state = self.mem;
        let state = match input.pedest {
            Some(Ok(d)) => brakes(d, input.speed),
            Some(Err(())) => Braking::NoBrake,
            None => previous_state,
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
        TimeoutPedestrian,
        TimeoutAeb,
        DelayAeb,
    }
    impl timer_stream::Timing for RuntimeTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::TimeoutPedestrian => std::time::Duration::from_millis(2000u64),
                T::TimeoutAeb => std::time::Duration::from_millis(500u64),
                T::DelayAeb => std::time::Duration::from_millis(10u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::TimeoutPedestrian => true,
                T::TimeoutAeb => true,
                T::DelayAeb => true,
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
    #[pin_project::pin_project(project = RuntimeLoopProj)]
    pub struct RuntimeLoop<St>
    where
        St: futures::Stream<Item = I>,
    {
        #[pin]
        input: St,
        runtime: Runtime,
    }
    impl<St> std::future::Future for RuntimeLoop<St>
    where
        St: futures::Stream<Item = I>,
    {
        type Output = ();

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<()> {
            let mut this = self.project();
            loop {
                if let Some(input) = std::task::ready!(this.input.as_mut().poll_next(cx)) {
                    this.runtime.handle_input(input)
                } else {
                    break;
                }
            }
            std::task::Poll::Ready(())
        }
    }
    impl Runtime {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let aeb = aeb_service::AebService::init(output, timer.clone());
            Runtime { aeb, timer }
        }
        pub fn run_loop<St>(self, init_instant: std::time::Instant, input: St) -> RuntimeLoop<St>
        where
            St: futures::Stream<Item = I>,
        {
            let mut runtime = self;
            let res = runtime.timer.try_send((T::TimeoutPedestrian, init_instant));
            if res.is_err() {
                panic!("timer channel is out of bound");
            }
            RuntimeLoop { input, runtime }
        }
        pub fn handle_input(&mut self, input: I) {
            match input {
                I::SpeedKmH(speed_km_h, instant) => {
                    self.aeb.handle_speed_km_h(instant, speed_km_h);
                }
                I::PedestrianL(pedestrian_l, instant) => {
                    self.aeb.handle_pedestrian_l(instant, pedestrian_l);
                }
                I::PedestrianR(pedestrian_r, instant) => {
                    self.aeb.handle_pedestrian_r(instant, pedestrian_r);
                }
                I::Timer(T::TimeoutPedestrian, instant) => {
                    self.aeb.handle_timeout_pedestrian(instant);
                }
                I::Timer(T::TimeoutAeb, instant) => {
                    self.aeb.handle_timeout_aeb(instant);
                }
                I::Timer(T::DelayAeb, instant) => {
                    self.aeb.handle_delay_aeb(instant);
                }
            }
        }
    }
    pub mod aeb_service {
        use super::*;
        use futures::{sink::SinkExt, stream::StreamExt};
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Context {
            pub brakes: Braking,
            pub speed_km_h: f64,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn get_braking_state_inputs(
                &self,
                pedestrian: Option<Result<f64, ()>>,
            ) -> BrakingStateInput {
                BrakingStateInput {
                    speed: self.speed_km_h,
                    pedest: pedestrian,
                }
            }
        }
        #[derive(Default)]
        pub struct AebServiceInputStore {
            pedestrian_r: Option<(f64, std::time::Instant)>,
            pedestrian_l: Option<(f64, std::time::Instant)>,
            timeout_pedestrian: Option<((), std::time::Instant)>,
            speed_km_h: Option<(f64, std::time::Instant)>,
        }
        impl AebServiceInputStore {
            pub fn not_empty(&self) -> bool {
                self.pedestrian_r.is_some()
                    || self.pedestrian_l.is_some()
                    || self.timeout_pedestrian.is_some()
                    || self.speed_km_h.is_some()
            }
        }
        pub struct AebService {
            context: Context,
            delayed: bool,
            input_store: AebServiceInputStore,
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
                let braking_state = BrakingStateState::init();
                AebService {
                    context,
                    delayed: true,
                    input_store: Default::default(),
                    braking_state,
                    output,
                    timer,
                }
            }
            pub fn handle_timeout_aeb(&mut self, instant: std::time::Instant) {
                self.reset_time_constrains(instant);
                let brakes = self
                    .braking_state
                    .step(self.context.get_braking_state_inputs(None));
                self.context.brakes = brakes;
                let brakes = self.context.brakes;
                self.send_brakes(brakes, instant);
            }
            pub fn handle_delay_aeb(&mut self, instant: std::time::Instant) {
                if self.input_store.not_empty() {
                    self.reset_time_constrains(instant);
                    self.handle_input_store(instant);
                } else {
                    self.delayed = true;
                }
            }
            pub fn handle_speed_km_h(&mut self, instant: std::time::Instant, speed_km_h: f64) {
                if self.delayed {
                    self.reset_time_constrains(instant);
                    self.context.speed_km_h = speed_km_h;
                } else {
                    let unique = self.input_store.speed_km_h.replace((speed_km_h, instant));
                    assert!(unique.is_none(), "speed_km_h changes too frequently");
                }
            }
            pub fn handle_pedestrian_l(&mut self, instant: std::time::Instant, pedestrian_l: f64) {
                if self.delayed {
                    self.reset_time_constrains(instant);
                    let x = pedestrian_l;
                    let pedestrian = Ok(x);
                    self.reset_timeout_pedestrian(instant);
                    let brakes = self
                        .braking_state
                        .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                    self.context.brakes = brakes;
                    let brakes = self.context.brakes;
                    self.send_brakes(brakes, instant);
                } else {
                    let unique = self
                        .input_store
                        .pedestrian_l
                        .replace((pedestrian_l, instant));
                    assert!(unique.is_none(), "pedestrian changes too frequently");
                }
            }
            pub fn handle_pedestrian_r(&mut self, instant: std::time::Instant, pedestrian_r: f64) {
                if self.delayed {
                    self.reset_time_constrains(instant);
                    let x = pedestrian_r;
                    let pedestrian = Ok(x);
                    self.reset_timeout_pedestrian(instant);
                    let brakes = self
                        .braking_state
                        .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                    self.context.brakes = brakes;
                    let brakes = self.context.brakes;
                    self.send_brakes(brakes, instant);
                } else {
                    let unique = self
                        .input_store
                        .pedestrian_r
                        .replace((pedestrian_r, instant));
                    assert!(unique.is_none(), "pedestrian changes too frequently");
                }
            }
            pub fn handle_timeout_pedestrian(&mut self, instant: std::time::Instant) {
                if self.delayed {
                    self.reset_time_constrains(instant);
                    let pedestrian = Err(());
                    self.reset_timeout_pedestrian(instant);
                    let brakes = self
                        .braking_state
                        .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                    self.context.brakes = brakes;
                    let brakes = self.context.brakes;
                    self.send_brakes(brakes, instant);
                } else {
                    let unique = self.input_store.timeout_pedestrian.replace(((), instant));
                    assert!(unique.is_none(), "pedestrian changes too frequently");
                }
            }
            #[inline]
            pub fn handle_input_store(&mut self, instant: std::time::Instant) {
                match (
                    self.input_store.speed_km_h.take(),
                    self.input_store.pedestrian_l.take(),
                    self.input_store.pedestrian_r.take(),
                    self.input_store.timeout_pedestrian.take(),
                ) {
                    (None, None, None, None) => unreachable!(),
                    (None, None, None, Some(((), instant_timeout_pedestrian))) => {
                        let pedestrian = Err(());
                        self.reset_timeout_pedestrian(instant_timeout_pedestrian);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (None, None, Some((pedestrian_r, instant_pedestrian_r)), None) => {
                        let x = pedestrian_r;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_r);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (
                        None,
                        None,
                        Some((pedestrian_r, instant_pedestrian_r)),
                        Some(((), instant_timeout_pedestrian)),
                    ) => {
                        let x = pedestrian_r;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_r);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (None, Some((pedestrian_l, instant_pedestrian_l)), None, None) => {
                        let x = pedestrian_l;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_l);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (
                        None,
                        Some((pedestrian_l, instant_pedestrian_l)),
                        None,
                        Some(((), instant_timeout_pedestrian)),
                    ) => {
                        let x = pedestrian_l;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_l);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (
                        None,
                        Some((pedestrian_l, instant_pedestrian_l)),
                        Some((pedestrian_r, instant_pedestrian_r)),
                        None,
                    ) => {
                        let x = pedestrian_l;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_l);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (
                        None,
                        Some((pedestrian_l, instant_pedestrian_l)),
                        Some((pedestrian_r, instant_pedestrian_r)),
                        Some(((), instant_timeout_pedestrian)),
                    ) => {
                        let x = pedestrian_l;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_l);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (Some((speed_km_h, instant_speed_km_h)), None, None, None) => {
                        self.context.speed_km_h = speed_km_h;
                    }
                    (
                        Some((speed_km_h, instant_speed_km_h)),
                        None,
                        None,
                        Some(((), instant_timeout_pedestrian)),
                    ) => {
                        self.context.speed_km_h = speed_km_h;
                        let pedestrian = Err(());
                        self.reset_timeout_pedestrian(instant_timeout_pedestrian);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (
                        Some((speed_km_h, instant_speed_km_h)),
                        None,
                        Some((pedestrian_r, instant_pedestrian_r)),
                        None,
                    ) => {
                        self.context.speed_km_h = speed_km_h;
                        let x = pedestrian_r;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_r);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (
                        Some((speed_km_h, instant_speed_km_h)),
                        None,
                        Some((pedestrian_r, instant_pedestrian_r)),
                        Some(((), instant_timeout_pedestrian)),
                    ) => {
                        self.context.speed_km_h = speed_km_h;
                        let x = pedestrian_r;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_r);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (
                        Some((speed_km_h, instant_speed_km_h)),
                        Some((pedestrian_l, instant_pedestrian_l)),
                        None,
                        None,
                    ) => {
                        self.context.speed_km_h = speed_km_h;
                        let x = pedestrian_l;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_l);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (
                        Some((speed_km_h, instant_speed_km_h)),
                        Some((pedestrian_l, instant_pedestrian_l)),
                        None,
                        Some(((), instant_timeout_pedestrian)),
                    ) => {
                        self.context.speed_km_h = speed_km_h;
                        let x = pedestrian_l;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_l);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (
                        Some((speed_km_h, instant_speed_km_h)),
                        Some((pedestrian_l, instant_pedestrian_l)),
                        Some((pedestrian_r, instant_pedestrian_r)),
                        None,
                    ) => {
                        self.context.speed_km_h = speed_km_h;
                        let x = pedestrian_l;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_l);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                    (
                        Some((speed_km_h, instant_speed_km_h)),
                        Some((pedestrian_l, instant_pedestrian_l)),
                        Some((pedestrian_r, instant_pedestrian_r)),
                        Some(((), instant_timeout_pedestrian)),
                    ) => {
                        self.context.speed_km_h = speed_km_h;
                        let x = pedestrian_l;
                        let pedestrian = Ok(x);
                        self.reset_timeout_pedestrian(instant_pedestrian_l);
                        let brakes = self
                            .braking_state
                            .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                        self.context.brakes = brakes;
                        let brakes = self.context.brakes;
                        self.send_brakes(brakes, instant);
                    }
                }
            }
            #[inline]
            pub fn send_brakes(&mut self, brakes: Braking, instant: std::time::Instant) {
                let res = self.output.try_send(O::Brakes(brakes, instant));
                if res.is_err() {
                    panic!("output channel is out of bound");
                }
            }
            #[inline]
            pub fn reset_timeout_pedestrian(&mut self, instant: std::time::Instant) {
                let res = self.timer.try_send((T::TimeoutPedestrian, instant));
                if res.is_err() {
                    panic!("timer channel is out of bound");
                }
            }
            #[inline]
            pub fn reset_time_constrains(&mut self, instant: std::time::Instant) {
                self.reset_delay_aeb(instant);
                self.reset_timeout_aeb(instant);
                self.delayed = false;
            }
            #[inline]
            pub fn reset_delay_aeb(&mut self, instant: std::time::Instant) {
                let res = self.timer.try_send((T::DelayAeb, instant));
                if res.is_err() {
                    panic!("timer channel is out of bound");
                }
            }
            #[inline]
            pub fn reset_timeout_aeb(&mut self, instant: std::time::Instant) {
                let res = self.timer.try_send((T::TimeoutAeb, instant));
                if res.is_err() {
                    panic!("timer channel is out of bound");
                }
            }
        }
    }
}
