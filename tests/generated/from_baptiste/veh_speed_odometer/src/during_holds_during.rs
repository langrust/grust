pub struct DuringHoldsDuringInput {
    pub condition: bool,
    pub duration_ms: i64,
    pub time_ms: i64,
}
pub struct DuringHoldsDuringState {
    mem_prev_still_holds: bool,
    mem_start_ms: i64,
}
impl DuringHoldsDuringState {
    pub fn init() -> DuringHoldsDuringState {
        DuringHoldsDuringState {
            mem_prev_still_holds: false,
            mem_start_ms: 0i64,
        }
    }
    pub fn step(&mut self, input: DuringHoldsDuringInput) -> bool {
        let prev_still_holds = self.mem_prev_still_holds;
        let start_during = !prev_still_holds && input.condition;
        let start_ms = if start_during { input.time_ms } else { self.mem_start_ms };
        let during = input.time_ms > start_ms + input.duration_ms;
        let still_holds = if start_during {
            true
        } else {
            prev_still_holds && input.condition
        };
        let holds_during = still_holds && during;
        self.mem_prev_still_holds = still_holds;
        self.mem_start_ms = start_ms;
        holds_during
    }
}
