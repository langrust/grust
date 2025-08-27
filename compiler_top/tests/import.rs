compiler_top::prelude! {}

#[test]
fn should_compile_import() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/import.rs"]

        use component utils::counter(res: bool, tick: unit?) -> (o: int);

        component test(tick: unit?) -> (y: int) {
            init stop = false;
            let stop: bool = y > 35;
            y = counter(last stop, tick);
        }
    };
    let (ast, mut ctx) = top.init();
    let tokens = compiler_top::into_token_stream_res(ast, &mut ctx).unwrap();
    if let Some(path) = ctx.conf.dump_code {
        compiler_top::dump_code(&path, &tokens).unwrap();
    }
}
