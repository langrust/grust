grust_compiler_top::prelude! {}

#[test]
fn should_compile_rising_edges() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/rising_edges.rs"]

        component rising_edges(a: int?, b: int?, v: int) -> (
            c: int,
            d: float,
            x: int?,
        ) {
            c = when { init => 0, w? => w, a? => a };
            d = when { init => 0., let _ = y? => 0.1 };
            let w: int? = when { v > 50 => emit v + (last c) };
            when {
                init => {
                    z = 0;
                }
                (a?, let e = b?, v > 50) => {
                    let z: int =  if v > 80 then e else a;
                    let y: unit? = emit ();
                }
                (v < 40, a?) if a != 0 => {
                    let z: int = 2;
                    x = emit 2;
                }
                let e = b? if e < 20=> {
                    x = emit 2;
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
