grust_compiler_top::prelude! {}

#[test]
fn should_compile_multiple_events() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/multiple_events.rs"]

        component multiple_events(a: int?, b: int?, v: int) -> (c: int, d: int) {
            c = z;
            d = when { init => 0, (a?, b?) => 10 * a + b };
            when {
                init => {
                    z = 0;
                    a_bis = 0;
                }
                (let a = a?, let b = b?) if v > 50 => {
                    let a_bis: int = a;
                    let z: int = last a_bis;
                }
                let a = a? => {
                    let a_bis: int = a;
                    let z: int = 2;
                }
                let b = b? => {
                    let z: int = if v > 50 then 3 else 4;
                }
            }
        }
    };
    let (ast, mut ctx) = top.init();
    let tokens = grust_compiler_top::into_token_stream_res(ast, &mut ctx).unwrap();
    if let Some(path) = ctx.conf.dump_code {
        grust_compiler_top::dump_code(&path, &tokens).unwrap();
    }
}
