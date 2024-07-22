compiler::prelude! {
    ast::Ast, conf
}

#[test]
fn should_compile_multiple_events() {
    let ast: Ast = syn::parse_quote! {
        #![dump = "tests/macro_outputs/multiple_events.rs"]

        component multiple_events(a: int?, b: int?, v: int) -> (c: int) {
            c = z;
            when {
                (let a = a?, let b = b?) => {
                    let z: int = if v > 50 then 1 else 0;
                },
                let a = a? => {
                    let z: int = 2;
                },
                let b = b? => {
                    let z: int = if v > 50 then 3 else 4;
                },
                otherwise => {
                    let z: int = 0 fby c;
                }
            }
        }
    };
    let tokens = compiler::into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        compiler::dump_code(&path, &tokens);
    }
}
