compiler_top::prelude! {}

#[test]
fn should_compile_para() {
    let ast: Ast = parse_quote! {
        #![dump = "tests/macro_outputs/para.rs", propag = "onchange", para, test]
        import event e0: int;
        export signal o1: int;

        component C1(e0: int?) -> (s2: int, e1: int?) {
            when {
                e0? if e0 > prev_s2 => {
                    s2 = e0;
                    e1 = emit e0 / (e0 - s2);
                }
                e0? => {
                    s2 = e0;
                }
            }
            let prev_s2: int = last s2;
        }

        component C2(e1: int?) -> (s3: int, e3: int?) {
            when {
                e1? => {
                    s3 = e1;
                }
                prev_s3 > 0 => {
                    s3 = prev_s3;
                    e3 = emit prev_s3;
                }
            }
            let prev_s3: int = last s3;
        }

        component C3(s2: int) -> (e2: int?) {
            e2 = when s2 > 1 then emit s2;
        }

        component C4(e2: int?) -> (s4: int) {
            s4 = when e2? then e2;
        }

        component C5(s4: int, s3: int, e3: int?) -> (o: int) {
            when {
                e3? => {
                    o = e3;
                }
                s4 <= 0 => {
                    o = prev_o*2;
                }
                s3 >= 0 => {
                    o = s3;
                }
            }
            let prev_o: int = last o;
        }

        service para_mess {
            let (signal s2: int, event e1: int) = C1(e0);
            let (signal s3: int, event e3: int) = C2(e1);
            let (event e2: int) = C3(s2);
            let (signal s4: int) = C4(e2);
            o1 = C5(s4, s3, e3);
        }
    };
    let tokens = compiler_top::into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        compiler_top::dump_code(&path, &tokens);
    }
}
