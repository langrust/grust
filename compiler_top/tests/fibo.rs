compiler_top::prelude! {}

pub mod module {
    pub fn add_isize(n: i64, m: i64) -> i64 {
        n + m
    }
}

#[test]
fn should_compile_fibo() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/fibo.rs"]

        fn module::add_isize(n: int, m: int) -> int;

        component next(i: int) -> (next_o: int) {
            init i = 1;
            next_o = add_isize(i, last i);
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
