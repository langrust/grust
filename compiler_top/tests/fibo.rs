compiler_top::prelude! {}

#[test]
fn should_compile_fibo() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/fibo.rs"]

        component next(i: int) -> (next_o: int) {
            init i = 1;
            next_o = i + last i;
        }

        component semi_fib(i: int) -> (o: int) {
            let next_o: int = next(i);
            init next_o = 0;
            o = last next_o;
        }

        component fib_call() -> (fib: int) {
            fib = semi_fib(fib);
        }

        component fib() -> (fib: int) {
            init fib = 1;
            init next_o = 0;
            let next_o: int = fib + last fib;
            fib = last next_o;
        }
    };
    let (ast, mut ctx) = top.init();
    let tokens = compiler_top::into_token_stream(ast, &mut ctx);
    if let Some(path) = ctx.conf.dump_code {
        compiler_top::dump_code(&path, &tokens).unwrap();
    }
}
