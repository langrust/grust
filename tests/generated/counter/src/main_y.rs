use crate::counter_o::*;
pub struct main_yInput {}
pub struct main_yState {
    mem_half: bool,
    mem_x: bool,
    counter_o_y: counter_oState,
}
impl main_yState {
    pub fn init() -> main_yState {
        main_yState {
            mem_half: true,
            mem_x: false,
            counter_o_y: counter_oState::init(),
        }
    }
    pub fn step(self, input: main_yInput) -> (main_yState, i64) {
        let half = self.mem_half;
        let x = self.mem_x;
        let (counter_o_y, y) = self
            .counter_o_y
            .step(counter_oInput {
                res: x,
                tick: half,
            });
        (
            main_yState {
                mem_half: !half,
                mem_x: (y > 35i64),
                counter_o_y,
            },
            y,
        )
    }
}
