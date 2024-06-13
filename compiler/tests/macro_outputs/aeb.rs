#[derive(Clone, Copy, Debug, PartialEq, Default)]
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
pub enum BrakingStateEvent {
    pedest(Result<f64, ()>),
    NoEvent,
}
pub struct BrakingStateInput {
    pub speed: f64,
    pub braking_state_event: BrakingStateEvent,
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
    pub fn step(&mut self, input: BrakingStateInput) -> Braking {
        let state = match input.braking_state_event {
            BrakingStateEvent::pedest(Ok(d)) => {
                let state = brakes(d, input.speed);
                state
            }
            BrakingStateEvent::pedest(Err(())) => {
                let state = Braking::NoBrake;
                state
            }
            _ => {
                let state = self.mem;
                state
            }
        };
        self.mem = state;
        state
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Context {
    pub brakes: Braking,
    pub speed_km_h: f64,
}
impl Context {
    fn init() -> Context {
        Default::default()
    }
    fn get_braking_state_inputs(&self, event: BrakingStateEvent) -> BrakingStateInput {
        BrakingStateInput {
            speed: self.speed_km_h,
            braking_state_event: event,
        }
    }
}
pub mod toto_service {
    use super::*;
    use futures::{sink::SinkExt, stream::StreamExt};
    use TotoServiceInput as I;
    use TotoServiceOutput as O;
    use TotoServiceTimer as T;
    pub enum TotoServiceTimer {
        timeout_fresh_ident,
    }
    impl TotoServiceTimer {
        pub fn get_timer(&self) -> i64 {
            match self {
                T::timeout_fresh_ident => 500u64 as i64,
            }
        }
    }
    pub enum TotoServiceInput {
        speed_km_h(f64, i64),
        pedestrian_l(f64, i64),
        pedestrian_r(f64, i64),
        timer(T, i64),
    }
    pub enum TotoServiceOutput {
        brakes(Braking),
    }
    pub struct TotoService {
        context: Context,
        braking_state: BrakingStateState,
        output: futures::channel::mpsc::Sender<O>,
        timer: futures::channel::mpsc::Sender<(T, i64)>,
    }
    impl TotoService {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, i64)>,
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
        pub async fn run_loop(self, input: impl futures::Stream<Item = I>) {
            tokio::pin!(input);
            let mut service = self;
            loop {
                tokio::select! { input = input . next () => if let Some (input) = input { match input { I :: speed_km_h (speed_km_h , timestamp) => service . handle_speed_km_h (timestamp , speed_km_h) . await , I :: pedestrian_l (pedestrian_l , timestamp) => service . handle_pedestrian_l (timestamp , pedestrian_l) . await , I :: pedestrian_r (pedestrian_r , timestamp) => service . handle_pedestrian_r (timestamp , pedestrian_r) . await , I :: timer (T :: timeout_fresh_ident , timestamp) => service . handle_timeout_fresh_ident (timestamp) . await , } } else { break ; } }
            }
        }
        async fn handle_speed_km_h(&mut self, timestamp: i64, speed_km_h: f64) {
            self.context.speed_km_h = speed_km_h;
        }
        async fn handle_pedestrian_l(&mut self, timestamp: i64, pedestrian_l: f64) {
            let pedestrian = Ok(pedestrian_l);
            self.timer
                .send((T::timeout_fresh_ident, timestamp))
                .await
                .unwrap();
            let brakes = self.braking_state.step(
                self.context
                    .get_braking_state_inputs(BrakingStateEvent::pedest(pedestrian)),
            );
            self.context.brakes = brakes;
            self.output
                .send(O::brakes(self.context.brakes.clone()))
                .await
                .unwrap();
        }
        async fn handle_pedestrian_r(&mut self, timestamp: i64, pedestrian_r: f64) {}
        async fn handle_timeout_fresh_ident(&mut self, timestamp: i64) {
            let pedestrian = Err(());
            self.timer
                .send((T::timeout_fresh_ident, timestamp))
                .await
                .unwrap();
            let brakes = self.braking_state.step(
                self.context
                    .get_braking_state_inputs(BrakingStateEvent::pedest(pedestrian)),
            );
            self.context.brakes = brakes;
            self.output
                .send(O::brakes(self.context.brakes.clone()))
                .await
                .unwrap();
        }
    }
}
