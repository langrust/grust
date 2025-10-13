grust_compiler_top::prelude! {}

#[test]
fn should_compile_two_comp_in_one_service() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/two_comp_in_one_service.rs", mode = test]

        use component utils::counter(res: bool, tick: unit?) -> (o: int);

        import event clock: unit;
        import signal reset: bool;
        export signal o1: int;
        export signal o2: int;

        service test @ [10, 3000] {
            o1 = counter(reset, clock);
            o2 = counter(reset, timeout(clock, 1000));
        }
    };
    let (ast, mut ctx) = top.init();
    let tokens = grust_compiler_top::into_token_stream_res(ast, &mut ctx).unwrap();
    if let Some(path) = ctx.conf.dump_code {
        grust_compiler_top::dump_code(&path, &tokens).unwrap();
    }
}
