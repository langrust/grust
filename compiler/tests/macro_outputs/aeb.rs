#[derive(Clone, Copy, PartialEq, Default)]
pub enum Braking {
    #[default]
    UrgentBrake,
    SoftBrake,
    NoBrake,
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
    # [requires (0. <= input . speed && input . speed < 50.)]
    # [ensures (forall < p : f64 > Some (Ok (p)) == input . pedest == > result != Braking :: NoBrake)]
    pub fn step(&mut self, input: BrakingStateInput) -> Braking {
        let state = match (input.pedest, input.pedest) {
            (Some(Ok(d)), _) => {
                let state = brakes(d, input.speed);
                state
            }
            (_, Some(Err(()))) => {
                let state = Braking::NoBrake;
                state
            }
            (_, _) => {
                let state = self.mem;
                state
            }
        };
        self.mem = state;
        state
    }
}
#[derive(Clone, Copy, PartialEq, Default)]
pub struct Context {
    pub brakes: Braking,
    pub speed_km_h: f64,
}
impl Context {
    fn init() -> Context {
        Default::default()
    }
    fn get_braking_state_inputs(&self, pedestrian: Option<Result<f64, ()>>) -> BrakingStateInput {
        BrakingStateInput {
            speed: self.speed_km_h,
            pedest: pedestrian,
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
        timeout_fresh_ident,
    }
    impl timer_stream::Timing for TotoServiceTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::timeout_fresh_ident => std::time::Duration::from_millis(500u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::timeout_fresh_ident => true,
            }
        }
    }
    pub enum TotoServiceInput {
        speed_km_h(f64, std::time::Instant),
        pedestrian_l(f64, std::time::Instant),
        pedestrian_r(f64, std::time::Instant),
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
                (I::speed_km_h(this, _), I::speed_km_h(other, _)) => this.eq(other),
                (I::pedestrian_l(this, _), I::pedestrian_l(other, _)) => this.eq(other),
                (I::pedestrian_r(this, _), I::pedestrian_r(other, _)) => this.eq(other),
                (I::timer(this, _), I::timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl TotoServiceInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::speed_km_h(_, instant) => *instant,
                I::pedestrian_l(_, instant) => *instant,
                I::pedestrian_r(_, instant) => *instant,
                I::timer(_, instant) => *instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    pub enum TotoServiceOutput {
        brakes(Braking, std::time::Instant),
    }
    pub struct TotoService {
        context: Context,
        braking_state: BrakingStateState,
        output: futures::channel::mpsc::Sender<O>,
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl TotoService {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> TotoService {
            let context = Context::init();
            let braking_state = BrakingStateState::init();
            TotoService {
                context,
                braking_state,
                output,
                timer,
            }
        }
        pub async fn run_loop(
            self,
            init_instant: std::time::Instant,
            input: impl futures::Stream<Item = I>,
        ) {
            tokio::pin!(input);
            let mut service = self;
            {
                let res = service
                    .timer
                    .send((T::timeout_fresh_ident, init_instant))
                    .await;
                if res.is_err() {
                    return;
                }
            }
            loop {
                tokio::select! { input = input . next () => if let Some (input) = input { match input { I :: pedestrian_r (pedestrian_r , instant) => service . handle_pedestrian_r (instant , pedestrian_r) . await , I :: pedestrian_l (pedestrian_l , instant) => service . handle_pedestrian_l (instant , pedestrian_l) . await , I :: timer (T :: timeout_fresh_ident , instant) => service . handle_timeout_fresh_ident (instant) . await , I :: speed_km_h (speed_km_h , instant) => service . handle_speed_km_h (instant , speed_km_h) . await } } else { break ; } }
            }
        }
        async fn handle_pedestrian_r(&mut self, instant: std::time::Instant, pedestrian_r: f64) {}
        async fn handle_pedestrian_l(&mut self, instant: std::time::Instant, pedestrian_l: f64) {
            let pedestrian = Ok(pedestrian_l);
            {
                let res = self.timer.send((T::timeout_fresh_ident, instant)).await;
                if res.is_err() {
                    return;
                }
            }
            let brakes = self
                .braking_state
                .step(self.context.get_braking_state_inputs(Some(pedestrian)));
            self.context.brakes = brakes;
            let brakes = self.context.brakes;
            {
                let res = self.output.send(O::brakes(brakes, instant)).await;
                if res.is_err() {
                    return;
                }
            }
        }
        async fn handle_timeout_fresh_ident(&mut self, instant: std::time::Instant) {
            let pedestrian = Err(());
            {
                let res = self.timer.send((T::timeout_fresh_ident, instant)).await;
                if res.is_err() {
                    return;
                }
            }
            let brakes = self
                .braking_state
                .step(self.context.get_braking_state_inputs(Some(pedestrian)));
            self.context.brakes = brakes;
            let brakes = self.context.brakes;
            {
                let res = self.output.send(O::brakes(brakes, instant)).await;
                if res.is_err() {
                    return;
                }
            }
        }
        async fn handle_speed_km_h(&mut self, instant: std::time::Instant, speed_km_h: f64) {
            self.context.speed_km_h = speed_km_h;
        }
    }
}
