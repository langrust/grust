use creusot_contracts::{ensures, logic, open, prelude, requires, DeepModel};
#[derive(prelude :: Clone, Copy, prelude :: PartialEq, DeepModel)]
pub enum Braking {
    UrgentBrake,
    SoftBrake,
    NoBrake,
}
#[requires(0i64 <= speed && speed < 50i64)]
#[ensures(result == speed * speed / 100i64)]
pub fn compute_soft_braking_distance(speed: i64) -> i64 {
    speed * speed / 100i64
}
#[requires(0i64 <= speed && speed < 50i64)]
#[ensures(result != Braking :: NoBrake)]
pub fn brakes(distance: i64, speed: i64) -> Braking {
    let braking_distance = compute_soft_braking_distance(speed);
    let response = if braking_distance < distance {
        Braking::SoftBrake
    } else {
        Braking::UrgentBrake
    };
    response
}
pub struct BrakingStateInput {
    pub pedest: Option<Result<i64, ()>>,
    pub speed: i64,
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
    #[requires(0i64 <= input.speed && input.speed < 50i64)]
    #[ensures(forall < p : i64 > Some(Ok(p)) == input.pedest == > result !=
    Braking :: NoBrake)]
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
