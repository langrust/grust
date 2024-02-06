use crate::functions::factorial;
pub struct MainFInput {}
pub struct MainFState {
    mem_n: i64,
}
impl MainFState {
    pub fn init() -> MainFState {
        MainFState { mem_n: 0i64 }
    }
    pub fn step(&mut self, input: MainFInput) -> i64 {
        let n = self.mem_n;
        let f = Expr::FunctionCall(
            parse_quote! {
                factorial(n)
            },
        );
        self.mem_n = n + 1i64;
        f
    }
}
