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
    }
    impl timer_stream::Timing for RuntimeTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::TimeoutPedestrian => std::time::Duration::from_millis(2000u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::TimeoutPedestrian => true,
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
            runtime
                .send_timer(T::TimeoutPedestrian, init_instant)
                .await?;
            while let Some(input) = input.next().await {
                match input {
                    I::SpeedKmH(speed_km_h, instant) => {
                        runtime.aeb.handle_speed_km_h(instant, speed_km_h).await?;
                    }
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
                    I::Timer(T::TimeoutPedestrian, instant) => {
                        runtime.aeb.handle_timeout_pedestrian(instant).await?;
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
        pub struct AebServiceStore {
            speed_km_h: Option<(f64, std::time::Instant)>,
            pedestrian_l: Option<(f64, std::time::Instant)>,
            pedestrian_r: Option<(f64, std::time::Instant)>,
            timeout_pedestrian: Option<((), std::time::Instant)>,
        }
        impl AebServiceStore {
            pub fn not_empty(&self) -> bool {
                self.speed_km_h.is_some()
                    || self.pedestrian_l.is_some()
                    || self.pedestrian_r.is_some()
                    || self.timeout_pedestrian.is_some()
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
            pub async fn handle_speed_km_h(
                &mut self,
                instant: std::time::Instant,
                speed_km_h: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.context.speed_km_h = speed_km_h;
                Ok(())
            }
            pub async fn handle_pedestrian_l(
                &mut self,
                instant: std::time::Instant,
                pedestrian_l: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                let x = pedestrian_l;
                let pedestrian = Ok(x);
                self.send_timer(T::TimeoutPedestrian, instant).await?;
                let brakes = self
                    .braking_state
                    .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                self.context.brakes = brakes;
                let brakes = self.context.brakes;
                self.send_output(O::Brakes(brakes, instant)).await?;
                Ok(())
            }
            pub async fn handle_pedestrian_r(
                &mut self,
                instant: std::time::Instant,
                pedestrian_r: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                let x = pedestrian_r;
                let pedestrian = Ok(x);
                self.send_timer(T::TimeoutPedestrian, instant).await?;
                let brakes = self
                    .braking_state
                    .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                self.context.brakes = brakes;
                let brakes = self.context.brakes;
                self.send_output(O::Brakes(brakes, instant)).await?;
                Ok(())
            }
            pub async fn handle_timeout_pedestrian(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                let pedestrian = Err(());
                self.send_timer(T::TimeoutPedestrian, instant).await?;
                let brakes = self
                    .braking_state
                    .step(self.context.get_braking_state_inputs(Some(pedestrian)));
                self.context.brakes = brakes;
                let brakes = self.context.brakes;
                self.send_output(O::Brakes(brakes, instant)).await?;
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
