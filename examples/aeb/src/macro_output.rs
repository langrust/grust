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
    pub fn step(&mut self, input: BrakingStateInput) -> Braking {
        let previous_state = self.mem;
        let state = match input.braking_state_event {
            BrakingStateEvent::pedest(Ok(d)) => brakes(d, input.speed),
            BrakingStateEvent::pedest(Err(())) => Braking::NoBrake,
            _ => previous_state,
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
    pub enum TotoServiceInput {
        speed_km_h(f64),
        pedestrian_l(f64),
        pedestrian_r(f64),
    }
    pub enum TotoServiceOutput {
        brakes(Braking),
    }
    pub struct TotoService {
        context: Context,
        braking_state: BrakingStateState,
        output: futures::channel::mpsc::Sender<O>,
    }
    impl TotoService {
        pub fn new(output: futures::channel::mpsc::Sender<O>) -> TotoService {
            let context = Context::init();
            let braking_state = BrakingStateState::init();
            TotoService {
                context,
                braking_state,
                output,
            }
        }
        pub async fn run_loop(self, input: impl futures::Stream<Item = I>) {
            tokio::pin!(input);
            let mut service = self;
            let timeout_fresh_ident = tokio::time::sleep_until(
                tokio::time::Instant::now() + tokio::time::Duration::from_millis(2000u64),
            );
            tokio::pin!(timeout_fresh_ident);
            loop {
                tokio::select! {
                    input = input.next() => if let Some(input) = input
                    {
                        match input
                        {
                            I :: speed_km_h(speed_km_h) =>
                            service.handle_speed_km_h(speed_km_h).await, I ::
                            pedestrian_l(pedestrian_l) =>
                            service.handle_pedestrian_l(pedestrian_l,
                            timeout_fresh_ident.as_mut()).await, I ::
                            pedestrian_r(pedestrian_r) =>
                            service.handle_pedestrian_r(pedestrian_r).await
                        }
                    } else { break; }, _ = timeout_fresh_ident.as_mut() =>
                    service.handle_timeout_fresh_ident(timeout_fresh_ident.as_mut()).await,
                }
            }
        }
        async fn handle_speed_km_h(&mut self, speed_km_h: f64) {
            self.context.speed_km_h = speed_km_h;
        }
        async fn handle_pedestrian_l(&mut self, instant: std::time::Instant, pedestrian_l: f64) {
            let flow_expression_fresh_ident = pedestrian_l;
            let pedestrian = Ok(flow_expression_fresh_ident);
            {
                let res = self.timer.send((T::timeout_fresh_ident, instant)).await;
                if res.is_err() {
                    return;
                }
            }
            let brakes = self.braking_state.step(
                self.context
                    .get_braking_state_inputs(BrakingStateEvent::pedest(pedestrian)),
            );
            self.context.brakes = brakes;
            {
                let res = self
                    .output
                    .send(O::brakes(self.context.brakes.clone(), instant))
                    .await;
                if res.is_err() {
                    return;
                }
            }
        }
        async fn handle_pedestrian_r(&mut self, instant: std::time::Instant, pedestrian_r: f64) {
            let flow_expression_fresh_ident = pedestrian_r;
            let pedestrian = Ok(flow_expression_fresh_ident);
            {
                let res = self.timer.send((T::timeout_fresh_ident, instant)).await;
                if res.is_err() {
                    return;
                }
            }
            let brakes = self.braking_state.step(
                self.context
                    .get_braking_state_inputs(BrakingStateEvent::pedest(pedestrian)),
            );
            self.context.brakes = brakes;
            {
                let res = self
                    .output
                    .send(O::brakes(self.context.brakes.clone(), instant))
                    .await;
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
            let brakes = self.braking_state.step(
                self.context
                    .get_braking_state_inputs(BrakingStateEvent::pedest(pedestrian)),
            );
            self.context.brakes = brakes;
            {
                let res = self
                    .output
                    .send(O::brakes(self.context.brakes.clone(), instant))
                    .await;
                if res.is_err() {
                    return;
                }
            }
        }
    }
}
