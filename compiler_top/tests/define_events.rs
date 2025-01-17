compiler_top::prelude! {}

#[test]
fn should_compile_define_events() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/define_events.rs"]

        component define_events(a: int?, b: int?, v: int) -> (
            c: int,
            d: float?,
            x: int?,
        ) {
            c = z;
            d = when { y? => emit 0.1 };
            when {
                init             => {
                    z = 0;
                }
                (a?, let e = b?) => {
                    let z: int =  if v > 50 then e else a;
                    let y: unit? = emit ();
                }
                let _ = a?       => {
                    x = emit 2;
                }
                let _ = b?       => {
                    let z: int = if v > 50 then 3 else 4;
                    x = emit 2;
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
