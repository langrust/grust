use creusot_contracts::{ensures, requires};
#[derive(Clone, Copy)]
pub enum Braking {
    UrgentBrake,
    SoftBrake,
    NoBrake,
    SoftBrake,
    UrgentBrake,
}
#[requires(0i64 <= speed && speed < 50i64)]
pub fn compute_soft_braking_distance(speed: i64) -> i64 {
    speed * speed / 100i64
}
#[requires(0i64 <= speed && speed < 50i64)]
pub fn brakes(distance: i64, speed: i64) -> Braking {
    let braking_distance = compute_soft_braking_distance(speed);
    let response = if braking_distance < distance {
        Braking::SoftBrake
    } else {
        Braking::UrgentBrake
    };
    response
}
pub enum BrakingStateEvent {
    pedest(Result<i64, ()>),
    NoEvent,
}
pub struct BrakingStateInput {
    pub speed: i64,
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
    #[requires(0i64 <= input.speed && input.speed < 50i64)]
    #[ensures(forall < p : i64 > BrakingStateEvent :: pedest(Ok(p)) ==
    input.braking_state_event == > result != Braking :: NoBrake)]
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
