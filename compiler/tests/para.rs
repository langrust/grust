compiler::prelude! {
    ast::Ast,
    conf,
}

#[test]
fn should_compile_para() {
    let ast: Ast = syn::parse_quote! {
        #![dump = "tests/macro_outputs/para.rs", propag = "onchange", para, test]
        import event e0: int;
        export signal o1: int;

        component C1(e0: int?) -> (s2: int, e1: int?) {
            when {
                e0? => {
                    s2 = e0;
                    e1 = when e0 > s2 then e0 / (e0 - s2);
                }
                otherwise => {
                    s2 = prev_s2;
                }
            }
            let prev_s2: int = 0 fby s2;
        }

        component C2(e1: int?) -> (s3: int, e3: int?) {
            when {
                e1? => {
                    s3 = e1;
                }
                prev_s3 > 0 => {
                    s3 = prev_s3;
                    e3 = prev_s3;
                }
                otherwise => {
                    s3 = prev_s3;
                }
            }
            let prev_s3: int = 0 fby s3;
        }

        component C3(s2: int) -> (e2: int?) {
            e2 = when s2 > 1 then s2;
        }

        component C4(e2: int?) -> (s4: int) {
            s4 = when e2? then e2 otherwise 0 fby s4;
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
                otherwise => {
                    o = prev_o;
                }
            }
            let prev_o: int = 0 fby o;
        }

        service para_mess {
            let (signal s2: int, event e1: int) = C1(e0);
            let (signal s3: int, event e3: int) = C2(e1);
            let (event e2: int) = C3(s2);
            let (signal s4: int) = C4(e2);
            o1 = C5(s4, s3, e3);
        }
    };
    let tokens = compiler::into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        compiler::dump_code(&path, &tokens);
    }
}
