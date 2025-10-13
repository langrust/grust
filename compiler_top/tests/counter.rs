compiler_top::prelude! {}

#[test]
fn should_compile_counter() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/counter.rs"]

        function add(x: int, y: int) -> int {
            let res: int = x + y;
            return res;
        }

        component counter(res: bool, tick: bool) -> (o: int) {
            init o = 0;
            o = if res then 0 else add(last o, inc);
            let inc: int = if tick then 1 else 0;
        }

        component test() -> (y: int) {
            init (stop, not_half) = (false, false);
            let stop: bool = y > 35;
            y = counter(last stop, half);
            let not_half: bool = !half;
            let half: bool = last not_half;
        }
    };
    let (ast, mut ctx) = top.init();
    let tokens = compiler_top::into_token_stream_res(ast, &mut ctx).unwrap();
    if let Some(path) = ctx.conf.dump_code {
        compiler_top::dump_code(&path, &tokens).unwrap();
    }
}
