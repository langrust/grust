use creusot_contracts::{ensures, requires};

pub struct OverSpeedAlertAlertInput {
    pub speed: i64,
    pub dt: i64,
}
pub struct OverSpeedAlertAlertState {
    mem_prev_alert: i64,
    mem_t_level: i64,
}
impl OverSpeedAlertAlertState {
    pub fn init() -> OverSpeedAlertAlertState {
        OverSpeedAlertAlertState {
            mem_prev_alert: 0i64,
            mem_t_level: 0i64,
        }
    }
    #[ensures(0i64<= result)]
    #[ensures(result<= 2i64)]
    #[requires(0i64<= input.speed)]
    #[requires(input.speed<= 200i64)]
    #[requires(0i64<input.dt)]
    #[requires(input.dt<= 10i64)]
    pub fn step(&mut self, input: OverSpeedAlertAlertInput) -> i64 {
        let t_level = self.mem_t_level;
        let alert = if (80i64 < input.speed && input.speed < 120i64)
            && (t_level >= 1000i64)
        {
            1i64
        } else {
            (if (120i64 <= input.speed) && (t_level >= 1000i64) { 2i64 } else { 0i64 })
        };
        let prev_alert = self.mem_prev_alert;
        let change = prev_alert != alert;
        self.mem_prev_alert = alert;
        self
            .mem_t_level = if change {
            input.dt
        } else {
            (if t_level < 1000i64 { t_level + input.dt } else { t_level })
        };
        alert
    }
}
