#![allow(warnings)]

use grust::grust;
mod macro_output1;
mod macro_output2;

grust! {
    #![dump = "examples/para_test/src/macro_output1.rs", component_para_threads]

    component test1(i: int) -> (next_o: int) {
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
}

grust! {
    #![dump = "examples/para_test/src/macro_output2.rs", component_para_rayon]

    component test2(i: int) -> (next_o: int) {
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
}
