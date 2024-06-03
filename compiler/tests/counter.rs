compiler::prelude! {
    ast::Ast, conf
}

#[test]
fn should_compile_counter() {
    let ast: Ast = syn::parse_quote! {
        #![dump = "tests/macro_outputs/counter.rs"]

        function add(x: int, y: int) -> int {
            let res: int = x + y;
            return res;
        }

        component counter(res: bool, tick: bool) -> (o: int) {
            o = if res then 0 else add(0 fby o, inc);
            let inc: int = if tick then 1 else 0;
        }

        component test() -> (y: int) {
            y = counter(false fby (y > 35), half);
            let half: bool = true fby !half;
        }
    };
    let tokens = compiler::into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        compiler::dump_code(&path, &tokens);
    }
}
