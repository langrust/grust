#![allow(warnings)]

grust::grust! {
    #![dump_graph = "grust/out/para_graph.json"]

    component test_custom_aux(i: int) -> (next_o: int) {
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

    component test_custom(i : int) -> (next_o: int) {
        let i1_1: int = (i - 54) * 2;
        let i1_2: int = (i + 54) * 2;
        let i1_3: int = test_custom_aux(i);

        let i2_1: int = test_custom_aux(i1_1 + i1_2 - i1_3);
        let i2_2: int = test_custom_aux(i1_2 - i1_2 + i1_3);

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
