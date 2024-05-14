use compiler::{ast::Ast, conf, dump_code, into_token_stream};

#[test]
fn should_compile_counter() {
    let ast: Ast = syn::parse_quote! {
        #![dump = "C:/Users/az03049/Documents/gitlab/langrust/grustine/compiler/tests/macro_outputs/counter.rs"]

        component counter(res: bool, tick: bool) -> (o: int) {
            o = if res then 0 else (0 fby o) + inc;
            let inc: int = if tick then 1 else 0;
        }

        component test() -> (y: int) {
            y = counter(false fby (y > 35), half).o;
            let half: bool = true fby !half;
        }
    };
    let tokens = into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        dump_code(&path, &tokens);
    }
}
