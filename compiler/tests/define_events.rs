compiler::prelude! {
    ast::Ast, conf
}

#[test]
fn should_compile_define_events() {
    let ast: Ast = syn::parse_quote! {
        #![dump = "tests/macro_outputs/define_events.rs"]

        component define_events(a: int?, b: int?, v: int) -> (
            c: int,
            d: float?,
            x: int?,
        ) {
            c = z;
            d = when let _ = y? then emit 0.1;
            when {
                (a?, let e = b?) => {
                    let z: int =  if v > 50 then e else a;
                    let y: unit? = emit ();
                }
                let _ = a? => {
                    let z: int = 2;
                    x = emit 2;
                }
                let _ = b? => {
                    let z: int = if v > 50 then 3 else 4;
                    x = emit 2;
                }
            }
        }
    };
    let tokens = compiler::into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        compiler::dump_code(&path, &tokens);
    }
}
