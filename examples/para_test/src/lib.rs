#![allow(warnings)]

mod macro_output_mixed;
mod macro_output_rayon1;
mod macro_output_rayon2;
mod macro_output_rayon3;
mod macro_output_threads;

grust::grust! {
    #![dump = "examples/para_test/src/macro_output_threads.rs", component_para_threads]

    component test_threads_aux(i: int) -> (next_o: int) {
        let i1: int = (i - 54) * 2;
        let i2: int = (i + 54) * 2;
        let i3: int = 7 * i;
        let i12: int = i1 + i2;
        let i23: int = i2 + i3;
        let i123: int = i12 + 2 * i3 + i23;
        match i {
            0 => {
                next_o = 1 + last i init 0;
            },
            7 => {
                next_o = i123;
            },
            _ => {
                next_o = i12;
            },
        }
    }

    component test_threads(i : int) -> (next_o: int) {
        let i1_1: int = (i - 54) * 2;
        let i1_2: int = (i + 54) * 2;
        let i1_3: int = test_threads_aux(i);

        let i2_1: int = test_threads_aux(i1_1 + i1_2 - i1_3);
        let i2_2: int = test_threads_aux(i1_2 - i1_2 + i1_3);

        match i {
            0 => {
                next_o = 1 + last i init 0;
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

grust::grust! {
    #![dump = "examples/para_test/src/macro_output_rayon1.rs", component_para_rayon1]

    component test_rayon1_aux(i: int) -> (next_o: int) {
        let i1: int = (i - 54) * 2;
        let i2: int = (i + 54) * 2;
        let i3: int = 7 * i;
        let i12: int = i1 + i2;
        let i23: int = i2 + i3;
        let i123: int = i12 + 2 * i3 + i23;
        match i {
            0 => {
                next_o = 1 + last i init 0;
            },
            7 => {
                next_o = i123;
            },
            _ => {
                next_o = i12;
            },
        }
    }

    component test_rayon1(i : int) -> (next_o: int) {
        let i1_1: int = (i - 54) * 2;
        let i1_2: int = (i + 54) * 2;
        let i1_3: int = test_rayon1_aux(i);

        let i2_1: int = test_rayon1_aux(i1_1 + i1_2 - i1_3);
        let i2_2: int = test_rayon1_aux(i1_2 - i1_2 + i1_3);

        match i {
            0 => {
                next_o = 1 + last i init 0;
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

grust::grust! {
    #![dump = "examples/para_test/src/macro_output_rayon2.rs", component_para_rayon2]

    component test_rayon2_aux(i: int) -> (next_o: int) {
        let i1: int = (i - 54) * 2;
        let i2: int = (i + 54) * 2;
        let i3: int = 7 * i;
        let i12: int = i1 + i2;
        let i23: int = i2 + i3;
        let i123: int = i12 + 2 * i3 + i23;
        match i {
            0 => {
                next_o = 1 + last i init 0;
            },
            7 => {
                next_o = i123;
            },
            _ => {
                next_o = i12;
            },
        }
    }

    component test_rayon2(i : int) -> (next_o: int) {
        let i1_1: int = (i - 54) * 2;
        let i1_2: int = (i + 54) * 2;
        let i1_3: int = test_rayon2_aux(i);

        let i2_1: int = test_rayon2_aux(i1_1 + i1_2 - i1_3);
        let i2_2: int = test_rayon2_aux(i1_2 - i1_2 + i1_3);

        match i {
            0 => {
                next_o = 1 + last i init 0;
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

grust::grust! {
    #![dump = "examples/para_test/src/macro_output_rayon3.rs", component_para_rayon3]

    component test_rayon3_aux(i: int) -> (next_o: int) {
        let i1: int = (i - 54) * 2;
        let i2: int = (i + 54) * 2;
        let i3: int = 7 * i;
        let i12: int = i1 + i2;
        let i23: int = i2 + i3;
        let i123: int = i12 + 2 * i3 + i23;
        match i {
            0 => {
                next_o = 1 + last i init 0;
            },
            7 => {
                next_o = i123;
            },
            _ => {
                next_o = i12;
            },
        }
    }

    component test_rayon3(i : int) -> (next_o: int) {
        let i1_1: int = (i - 54) * 2;
        let i1_2: int = (i + 54) * 2;
        let i1_3: int = test_rayon3_aux(i);

        let i2_1: int = test_rayon3_aux(i1_1 + i1_2 - i1_3);
        let i2_2: int = test_rayon3_aux(i1_2 - i1_2 + i1_3);

        match i {
            0 => {
                next_o = 1 + last i init 0;
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

grust::grust! {
    #![dump = "examples/para_test/src/macro_output_mixed.rs", component_para_mixed]

    component test_mixed_aux(i: int) -> (next_o: int) {
        let i1: int = (i - 54) * 2;
        let i2: int = (i + 54) * 2;
        let i3: int = 7 * i;
        let i12: int = i1 + i2;
        let i23: int = i2 + i3;
        let i123: int = i12 + 2 * i3 + i23;
        match i {
            0 => {
                next_o = 1 + last i init 0;
            },
            7 => {
                next_o = i123;
            },
            _ => {
                next_o = i12;
            },
        }
    }

    component test_mixed(i : int) -> (next_o: int) {
        let i1_1: int = (i - 54) * 2;
        let i1_2: int = (i + 54) * 2;
        let i1_3: int = test_mixed_aux(i);

        let i2_1: int = test_mixed_aux(i1_1 + i1_2 - i1_3);
        let i2_2: int = test_mixed_aux(i1_2 - i1_2 + i1_3);

        match i {
            0 => {
                next_o = 1 + last i init 0;
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
