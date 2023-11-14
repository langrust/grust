use crate::countero::*;
pub struct mainyInput {}
pub struct mainyState {
    memhalf: bool,
    memx: bool,
    counteroy: counteroState,
}
impl mainyState {
    pub fn init() -> mainyState {
        mainyState {
            memhalf: true,
            memx: false,
            counteroy: counteroState::init(),
        }
    }
    pub fn step(self, input: mainyInput) -> (mainyState, i64) {
        let half = self.memhalf;
        let x = self.memx;
        let (counteroy, y) = self
            .counteroy
            .step(counteroInput {
                res: x,
                tick: half,
            });
        (
            mainyState {
                memhalf: !half,
                memx: (y > 35i64),
                counteroy,
            },
            y,
        )
    }
}
