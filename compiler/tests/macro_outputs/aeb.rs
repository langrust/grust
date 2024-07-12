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
