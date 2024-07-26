pub struct RisingEdgeInput {
    pub test: bool,
}
pub struct RisingEdgeState {
    mem: bool,
}
impl RisingEdgeState {
    pub fn init() -> RisingEdgeState {
        RisingEdgeState { mem: false }
    }
    pub fn step(&mut self, input: RisingEdgeInput) -> bool {
        let res = input.test && !self.mem;
        self.mem = input.test;
        res
    }
}

#[cfg(test)]
mod test {
    use super::{RisingEdgeInput, RisingEdgeState};

    #[test]
    fn should_return_the_value_of_the_input_at_first() {
        // false as input
        let mut rising_edge = RisingEdgeState::init();
        let input = RisingEdgeInput { test: false };
        assert!(!rising_edge.step(input));

        // true as input
        let mut rising_edge = RisingEdgeState::init();
        let input = RisingEdgeInput { test: true };
        assert!(rising_edge.step(input));
    }

    #[test]
    fn should_detect_rising_edge() {
        let inputs = [false, true];

        let mut rising_edge = RisingEdgeState::init();
        let mut output = false;
        for test in inputs {
            output = rising_edge.step(RisingEdgeInput { test });
        }

        assert!(output)
    }

    #[test]
    fn should_not_detect_falling_edge() {
        let inputs = [true, false];

        let mut rising_edge = RisingEdgeState::init();
        let mut output = true;
        for test in inputs {
            output = rising_edge.step(RisingEdgeInput { test });
        }

        assert!(!output)
    }

    #[test]
    fn should_not_detect_high_flat() {
        let inputs = [true, true];

        let mut rising_edge = RisingEdgeState::init();
        let mut output = true;
        for test in inputs {
            output = rising_edge.step(RisingEdgeInput { test });
        }

        assert!(!output)
    }

    #[test]
    fn should_not_detect_low_flat() {
        let inputs = [false, false];

        let mut rising_edge = RisingEdgeState::init();
        let mut output = true;
        for test in inputs {
            output = rising_edge.step(RisingEdgeInput { test });
        }

        assert!(!output)
    }
}
