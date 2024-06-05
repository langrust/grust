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
pub struct TotoService {
    context: Context,
    braking_state: BrakingStateState,
    speed_km_h_channel: tokio::sync::mpsc::Receiver<f64>,
    pedestrian_l_channel: tokio::sync::mpsc::Receiver<f64>,
    pedestrian_r_channel: tokio::sync::mpsc::Receiver<f64>,
    brakes_channel: tokio::sync::mpsc::Sender<Braking>,
}
impl TotoService {
    fn new(
        speed_km_h_channel: tokio::sync::mpsc::Receiver<f64>,
        pedestrian_l_channel: tokio::sync::mpsc::Receiver<f64>,
        pedestrian_r_channel: tokio::sync::mpsc::Receiver<f64>,
        brakes_channel: tokio::sync::mpsc::Sender<Braking>,
    ) -> TotoService {
        let braking_state = BrakingStateState::init();
        let context = Context::init();
        TotoService {
            context,
            braking_state,
            speed_km_h_channel,
            pedestrian_l_channel,
            pedestrian_r_channel,
            brakes_channel,
        }
    }
    pub async fn run_loop(
        speed_km_h_channel: tokio::sync::mpsc::Receiver<f64>,
        pedestrian_l_channel: tokio::sync::mpsc::Receiver<f64>,
        pedestrian_r_channel: tokio::sync::mpsc::Receiver<f64>,
        brakes_channel: tokio::sync::mpsc::Sender<Braking>,
    ) {
        let mut service = TotoService::new(
            speed_km_h_channel,
            pedestrian_l_channel,
            pedestrian_r_channel,
            brakes_channel,
        );
        let timeout_fresh_ident = tokio::time::sleep_until(
            tokio::time::Instant::now() + tokio::time::Duration::from_millis(500u64),
        );
        tokio::pin!(timeout_fresh_ident);
        loop {
            tokio::select! {
                speed_km_h = service.speed_km_h_channel.recv() =>
                service.handle_speed_km_h(speed_km_h.unwrap()).await,
                pedestrian_l = service.pedestrian_l_channel.recv() =>
                service.handle_pedestrian_l(pedestrian_l.unwrap(),
                timeout_fresh_ident.as_mut()).await, pedestrian_r =
                service.pedestrian_r_channel.recv() =>
                service.handle_pedestrian_r(pedestrian_r.unwrap()).await, _ =
                timeout_fresh_ident.as_mut() =>
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
