compiler::prelude! {
    ast::Ast, conf
}

#[test]
fn should_compile_fibo() {
    let ast: Ast = syn::parse_quote! {
        #![dump = "tests/macro_outputs/fibo.rs"]

        component next(i: int) -> (next_o: int) {
            next_o = i + last i init 1;
        }

        component semi_fib(i: int) -> (o: int) {
            let next_o: int = next(i);
            o = last next_o;
        }

        component fib_call() -> (fib: int) {
            fib = semi_fib(fib);
        }

        component fib() -> (fib: int) {
            let next_o: int = fib + last fib init 1;
            fib = last next_o;
        }
    };
    let tokens = compiler::into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        compiler::dump_code(&path, &tokens);
    }
}
