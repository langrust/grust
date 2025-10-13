#![allow(warnings)]

mod align {
    grust::grust! {
        #![component_para (2, 6, 20), align, dump = "grust/out/aligned.rs"]

        component aux(i: int) -> (next_o: int) {
            let i1: int = (i - 54) * 2;
            let i2: int = (i + 54) * 2;
            let i3: int = 7 * i;
            let i12: int = i1 + i2;
            let i23: int = i2 + i3;
            let i123: int = i12 + 2 * i3 + i23;
            init i = 0;
            match i {
                0 => {
                    next_o = 1 + last i;
                },
                7 => {
                    next_o = i123;
                },
                _ => {
                    next_o = i12;
                },
            }
        }

        component test(i : int) -> (next_o: int) {
            let i1_1: int = (i - 54) * 2;
            let i1_2: int = (i + 54) * 2;
            let i1_3: int = aux(i);

            let i2_1: int = aux(i1_1 + i1_2 - i1_3);
            let i2_2: int = aux(i1_2 - i1_2 + i1_3);

            init i = 0;
            match i {
                0 => {
                    next_o = 1 + last i;
                },
                7 => {
                    next_o = i2_1;
                },
                _ => {
                    next_o = i2_2;
                },
            }
        }
    }
}

mod no_align {
    grust::grust! {
        #![component_para (2, 6, 20), dump = "grust/out/not_aligned.rs"]

        component aux(i: int) -> (next_o: int) {
            let i1: int = (i - 54) * 2;
            let i2: int = (i + 54) * 2;
            let i3: int = 7 * i;
            let i12: int = i1 + i2;
            let i23: int = i2 + i3;
            let i123: int = i12 + 2 * i3 + i23;
            init i = 0;
            match i {
                0 => {
                    next_o = 1 + last i;
                },
                7 => {
                    next_o = i123;
                },
                _ => {
                    next_o = i12;
                },
            }
        }

        component test(i : int) -> (next_o: int) {
            let i1_1: int = (i - 54) * 2;
            let i1_2: int = (i + 54) * 2;
            let i1_3: int = aux(i);

            let i2_1: int = aux(i1_1 + i1_2 - i1_3);
            let i2_2: int = aux(i1_2 - i1_2 + i1_3);

            init i = 0;
            match i {
                0 => {
                    next_o = 1 + last i;
                },
                7 => {
                    next_o = i2_1;
                },
                _ => {
                    next_o = i2_2;
                },
            }
        }
    }
}

#[test]
fn memory_alignment() {
    assert_eq!(64, std::mem::align_of::<align::TestState>());
    assert_eq!(64, std::mem::align_of::<align::AuxState>());
    assert_eq!(256, std::mem::size_of::<align::TestState>());
    assert_eq!(64, std::mem::size_of::<align::AuxState>());

    assert_eq!(8, std::mem::align_of::<no_align::TestState>());
    assert_eq!(8, std::mem::align_of::<no_align::AuxState>());
    assert_eq!(32, std::mem::size_of::<no_align::TestState>());
    assert_eq!(8, std::mem::size_of::<no_align::AuxState>());
}
