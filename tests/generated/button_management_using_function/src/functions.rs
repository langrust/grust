pub fn reset_state_management(
    button_state: Button,
    counter: i64,
    reset_limit_1: i64,
) -> ResetState {
    let result = match button_state {
        Button::Released => ResetState::Confirmed,
        Button::Pressed if counter >= reset_limit_1 => ResetState::InProgress,
        _ => ResetState::Confirmed,
    };
    result
}
