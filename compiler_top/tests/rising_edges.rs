compiler_top::prelude! {}

#[test]
fn should_compile_rising_edges() {
    let ast: Ast = parse_quote! {
        #![dump = "tests/macro_outputs/rising_edges.rs"]

        component rising_edges(a: int?, b: int?, v: int) -> (
            c: int,
            d: float,
            x: int?,
        ) {
            c = when { init => 0, a? => a };
            d = when { init => 0., let _ = y? => 0.1 };
            let w: int? = when { v > 50 => emit v + (last c) };
            when {
                init => {
                    let z: int = 0;
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
    let tokens = compiler_top::into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        compiler_top::dump_code(&path, &tokens);
    }
}
