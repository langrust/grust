compiler_top::prelude! {}

#[test]
fn should_compile_multiple_events() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/multiple_events.rs"]

        component multiple_events(a: int?, b: int?, v: int) -> (c: int, d: int) {
            c = z;
            d = when { init => 0, (a?, b?) => 10 * a + b };
            when {
                init => {
                    let z: int = 0;
                }
                (let a = a?, let b = b?) if v > 50 => {
                    let z: int = 1;
                }
                let a = a? => {
                    let z: int = 2;
                }
                let b = b? => {
                    let z: int = if v > 50 then 3 else 4;
                }
            }
        }
    };
    let (ast, mut ctx) = top.init();
    let tokens = compiler_top::into_token_stream(ast, &mut ctx);
    if let Some(path) = ctx.conf.dump_code {
        compiler_top::dump_code(&path, &tokens).unwrap();
    }
}
