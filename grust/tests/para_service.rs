#![allow(warnings)]

/// The system implements the following graph:
///
/// ```text
///        |    |--s2-->| C3 |--e2-->| C4 |--s4-->|    |
/// --e0-->| C1 |                                 |    |
///        |    |--e1-->|    |--------s3--------->| C5 |--o-->
///                     | C2 |                    |    |
///                     |    |--------e3--------->|    |
/// ```
mod para {
    use grust::grust;

    grust! {
        #![service_para, mode = test, dump = "grust/out/para_service.rs"]

        import event e0: int;
        export signal o1: int;

        component C1(e0: int?) -> (s2: int, e1: int?) {
            when {
                init => {
                    s2 = 0;
                }
                e0? if e0 > prev_s2 => {
                    s2 = e0;
                    e1 = emit e0 / (e0 - prev_s2);
                }
                e0? => {
                    s2 = e0;
                }
            }
            let prev_s2: int = last s2;
        }

        component C2(e1: int?) -> (s3: int, e3: int?) {
            when {
                init => {
                    s3 = 0;
                }
                e1? if e1 > 1 => {
                    s3 = e1;
                    e3 = emit (last s3);
                }
                e1? => {
                    s3 = e1;
                }
            }
        }

        component C3(s2: int) -> (e2: int?) {
            e2 = when { s2 > 1 => emit s2 };
        }

        component C4(e2: int?) -> (s4: int) {
            s4 = when { init => 0, e2? => e2 };
        }

        component C5(s4: int, s3: int, e3: int?) -> (o: int) {
            o = when {
                init => 0,
                e3? => e3,
                s4 > 0 => s4*2,
                s3 >= 0 => s3,
            };
        }

        service para_mess @ [10, 3000] {
            let (signal s2: int, event e1: int) = C1(e0);
            let (signal s3: int, event e3: int) = C2(e1);
            let event e2: int = C3(s2);
            let signal s4: int = C4(e2);
            o1 = C5(s4, s3, e3);
        }
    }
}
