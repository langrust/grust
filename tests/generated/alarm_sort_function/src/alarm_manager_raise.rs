use crate::functions::sorting;
pub struct AlarmManagerRaiseInput {
    pub alarms: [Alarm; 10usize],
}
pub struct AlarmManagerRaiseState {}
impl AlarmManagerRaiseState {
    pub fn init() -> AlarmManagerRaiseState {
        AlarmManagerRaiseState {}
    }
    pub fn step(&mut self, input: AlarmManagerRaiseInput) -> [Alarm; 10usize] {
        let raise = Expr::FunctionCall(
            parse_quote! {
                sorting(input.alarms)
            },
        );
        raise
    }
}
