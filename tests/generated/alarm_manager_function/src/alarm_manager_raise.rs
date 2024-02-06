use crate::functions::alarms_processing;
pub struct AlarmManagerRaiseInput {
    pub alarms: [Alarm; 10usize],
}
pub struct AlarmManagerRaiseState {}
impl AlarmManagerRaiseState {
    pub fn init() -> AlarmManagerRaiseState {
        AlarmManagerRaiseState {}
    }
    pub fn step(&mut self, input: AlarmManagerRaiseInput) -> [bool; 10usize] {
        let raise = Expr::FunctionCall(
            parse_quote! {
                alarms_processing(input.alarms)
            },
        );
        raise
    }
}
