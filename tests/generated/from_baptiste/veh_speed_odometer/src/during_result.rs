pub struct DuringResultInput {
    pub condition: bool,
    pub duration_ms: i64,
    pub dt_ms: i64,
}
pub struct DuringResultState {
    mem_prev_time_ms: i64,
}
impl DuringResultState {
    pub fn init() -> DuringResultState {
        DuringResultState {
            mem_prev_time_ms: 0i64,
        }
    }
    pub fn step(&mut self, input: DuringResultInput) -> bool {
        let prev_time_ms = self.mem_prev_time_ms;
        let time_ms = if input.condition { prev_time_ms + input.dt_ms } else { 0i64 };
        let result = time_ms > input.duration_ms;
        self.mem_prev_time_ms = time_ms;
        result
    }
}
