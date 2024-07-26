compiler::prelude! {
    ast::Ast, conf
}

#[test]
fn should_compile_rising_edges() {
    let ast: Ast = syn::parse_quote! {
        #![dump = "tests/macro_outputs/rising_edges.rs"]

        import component grust::std::rising_edge: (test: bool) -> (res: bool);

        component rising_edges(a: int?, b: int?, v: int) -> (
            c: int,
            d: float,
            x: int?,
        ) {
            c = when a? then a otherwise z;
            d = when let _ = y? then 0.1 otherwise 0.2;
            when {
                (a?, let e = b?, v > 50) => {
                    let z: int =  if v > 80 then e else a;
                    let y: unit? = ();
                }
                (v < 40, a?) if a != 0 => {
                    let z: int = 2;
                    x = 2;
                }
                let e = b? => {
                    let z: int = if v > 50 then 3 else 4;
                    x = when e < 20 then 2;
                }
                otherwise => {
                    let z: int = when v > 50 then v + (0 fby c) otherwise 0;
                }
            }
        }
    };
    let tokens = compiler::into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        compiler::dump_code(&path, &tokens);
    }
}
