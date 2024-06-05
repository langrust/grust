use std::default;

use futures::StreamExt;
use priority_stream::{prio_stream, PrioStream};

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
pub enum ToToServiceInputs {
    speed_km_h(f64),
    pedestrian_l(f64),
    pedestrian_r(f64),
    timeout_fresh_ident,
}
pub struct TotoService {
    context: Context,
    braking_state: BrakingStateState,
    brakes_channel: tokio::sync::mpsc::Sender<Braking>,
}
impl TotoService {
    fn new(brakes_channel: tokio::sync::mpsc::Sender<Braking>) -> TotoService {
        let braking_state = BrakingStateState::init();
        let context = Context::init();
        TotoService {
            context,
            braking_state,
            brakes_channel,
        }
    }
    pub async fn run_loop<S: futures::Stream<Item = ToToServiceInputs>>(
        interface_stream: S,
        brakes_channel: tokio::sync::mpsc::Sender<Braking>,
    ) {
        tokio::pin!(interface_stream);
        let mut service = TotoService::new(brakes_channel);
        let timeout_fresh_ident = tokio::time::sleep_until(
            tokio::time::Instant::now() + tokio::time::Duration::from_millis(500u64),
        );
        tokio::pin!(timeout_fresh_ident);
        loop {
            tokio::select! {
                input = interface_stream.next() => match input.unwrap() {
                    (ToToServiceInputs::speed_km_h(speed_km_h)) => {
                        service.handle_speed_km_h(speed_km_h).await
                    }
                    (ToToServiceInputs::pedestrian_l(pedestrian_l)) => {
                        service
                            .handle_pedestrian_l(pedestrian_l, timeout_fresh_ident.as_mut())
                            .await
                    }
                    (ToToServiceInputs::pedestrian_r(pedestrian_r)) => service.handle_pedestrian_r(pedestrian_r).await,
                },
                _ = timeout_fresh_ident.as_mut() =>
                service.handle_timeout_fresh_ident(timeout_fresh_ident.as_mut()).await,
            }
        }
    }
    async fn handle_speed_km_h(&mut self, speed_km_h: f64) {
        self.context.speed_km_h = speed_km_h;
    }
    async fn handle_pedestrian_l(
        &mut self,
        pedestrian_l: f64,
        timeout_fresh_ident: std::pin::Pin<&mut tokio::time::Sleep>,
    ) {
        let pedestrian = Ok(pedestrian_l);
        timeout_fresh_ident
            .reset(tokio::time::Instant::now() + tokio::time::Duration::from_millis(500u64));
        let brakes = self.braking_state.step(
            self.context
                .get_braking_state_inputs(BrakingStateEvent::pedest(pedestrian)),
        );
        self.context.brakes = brakes;
    }
    async fn handle_pedestrian_r(&mut self, pedestrian_r: f64) {}
    async fn handle_timeout_fresh_ident(
        &mut self,
        timeout_fresh_ident: std::pin::Pin<&mut tokio::time::Sleep>,
    ) {
        let pedestrian = Err(());
        timeout_fresh_ident
            .reset(tokio::time::Instant::now() + tokio::time::Duration::from_millis(500u64));
        let brakes = self.braking_state.step(
            self.context
                .get_braking_state_inputs(BrakingStateEvent::pedest(pedestrian)),
        );
        self.context.brakes = brakes;
    }
}

#[derive(Default)]
pub enum Interface {
    #[default]
    none,
    speed_km_h(f64),
    pedestrian_l(f64),
    pedestrian_r(f64),
}
impl Interface {
    fn order(i1: &Interface, i2: &Interface) -> std::cmp::Ordering {
        match (i1, i2) {
            (Interface::speed_km_h(_), Interface::speed_km_h(_)) => todo!(),
            (Interface::speed_km_h(_), Interface::pedestrian_l(_)) => todo!(),
            (Interface::speed_km_h(_), Interface::pedestrian_r(_)) => todo!(),
            (Interface::pedestrian_l(_), Interface::speed_km_h(_)) => todo!(),
            (Interface::pedestrian_l(_), Interface::pedestrian_l(_)) => todo!(),
            (Interface::pedestrian_l(_), Interface::pedestrian_r(_)) => todo!(),
            (Interface::pedestrian_r(_), Interface::speed_km_h(_)) => todo!(),
            (Interface::pedestrian_r(_), Interface::pedestrian_l(_)) => todo!(),
            (Interface::pedestrian_r(_), Interface::pedestrian_r(_)) => todo!(),
            (Interface::none, Interface::none) => todo!(),
            (Interface::none, Interface::speed_km_h(_)) => todo!(),
            (Interface::none, Interface::pedestrian_l(_)) => todo!(),
            (Interface::none, Interface::pedestrian_r(_)) => todo!(),
            (Interface::speed_km_h(_), Interface::none) => todo!(),
            (Interface::pedestrian_l(_), Interface::none) => todo!(),
            (Interface::pedestrian_r(_), Interface::none) => todo!(),
        }
    }
}
pub struct Runtime<S>
where
    S: futures::Stream<Item = Interface>,
{
    toto_service: TotoService,
    prio_stream: PrioStream<S, fn(i1: &Interface, i2: &Interface) -> std::cmp::Ordering, 3>,
    timeout_fresh_ident: tokio::time::Sleep,
}
impl<S> Runtime<S>
where
    S: futures::Stream<Item = Interface>,
{
    fn from_io_stream(stream: S, brakes_channel: tokio::sync::mpsc::Sender<Braking>) -> Self {
        let toto_service = TotoService::new(brakes_channel);
        let prio_stream: PrioStream<S, fn(&Interface, &Interface) -> std::cmp::Ordering, 3> = prio_stream(stream, Interface::order);
        let timeout_fresh_ident = tokio::time::sleep_until(
            tokio::time::Instant::now() + tokio::time::Duration::from_millis(500u64),
        );
        Runtime {
            toto_service,
            prio_stream,
            timeout_fresh_ident,
        }
    }
}
