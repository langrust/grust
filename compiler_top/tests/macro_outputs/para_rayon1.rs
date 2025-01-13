pub struct TestRayon1AuxInput {
    pub i: i64,
}
pub struct TestRayon1AuxState {
    last_i: i64,
}
impl TestRayon1AuxState {
    pub fn init() -> TestRayon1AuxState {
        TestRayon1AuxState { last_i: 0i64 }
    }
    pub fn step(&mut self, input: TestRayon1AuxInput) -> i64 {
        let ((i1, i3, i2), ()) = std::thread::scope(|reserved_grust_thread_scope| {
            let (i1, i3, i2) = {
                let (
                    reserved_grust_rayon_opt_var_0,
                    reserved_grust_rayon_opt_var_1,
                    reserved_grust_rayon_opt_var_2,
                ) = {
                    #[allow(unused_imports)]
                    use grust::rayon::prelude::*;
                    (0 .. 3usize) . into_par_iter () . map (| idx : usize | match idx { 0usize => (Some ({ (input . i - 54i64) * 2i64 }) , None , None) , 1usize => (None , Some ({ 7i64 * input . i }) , None) , 2usize => (None , None , Some ({ (input . i + 54i64) * 2i64 })) , idx => unreachable ! ("fatal error in rayon branches, illegal index `{}`" , idx ,) , }) . reduce (| | (None , None , None ,) , | (reserved_grust_rayon_opt_var_0 , reserved_grust_rayon_opt_var_1 , reserved_grust_rayon_opt_var_2) , (reserved_grust_rayon_opt_var_0_rgt , reserved_grust_rayon_opt_var_1_rgt , reserved_grust_rayon_opt_var_2_rgt) | (match (reserved_grust_rayon_opt_var_0 , reserved_grust_rayon_opt_var_0_rgt) { (None , None) => None , (Some (val) , None) | (None , Some (val)) => Some (val) , (Some (_) , Some (_)) => unreachable ! ("fatal error in rayon reduce operation, found two values") } , match (reserved_grust_rayon_opt_var_1 , reserved_grust_rayon_opt_var_1_rgt) { (None , None) => None , (Some (val) , None) | (None , Some (val)) => Some (val) , (Some (_) , Some (_)) => unreachable ! ("fatal error in rayon reduce operation, found two values") } , match (reserved_grust_rayon_opt_var_2 , reserved_grust_rayon_opt_var_2_rgt) { (None , None) => None , (Some (val) , None) | (None , Some (val)) => Some (val) , (Some (_) , Some (_)) => unreachable ! ("fatal error in rayon reduce operation, found two values") }))
                };
                (
                    reserved_grust_rayon_opt_var_0.expect(
                        "unreachable: fatal error in final rayon unwrap, unexpected `None` value",
                    ),
                    reserved_grust_rayon_opt_var_1.expect(
                        "unreachable: fatal error in final rayon unwrap, unexpected `None` value",
                    ),
                    reserved_grust_rayon_opt_var_2.expect(
                        "unreachable: fatal error in final rayon unwrap, unexpected `None` value",
                    ),
                )
            };
            let () = ();
            ((i1, i3, i2), ())
        });
        let ((i12, i23), ()) = std::thread::scope(|reserved_grust_thread_scope| {
            let (i12, i23) = {
                let (reserved_grust_rayon_opt_var_0, reserved_grust_rayon_opt_var_1) = {
                    #[allow(unused_imports)]
                    use grust::rayon::prelude::*;
                    (0 .. 2usize) . into_par_iter () . map (| idx : usize | match idx { 0usize => (Some ({ i1 + i2 }) , None) , 1usize => (None , Some ({ i2 + i3 })) , idx => unreachable ! ("fatal error in rayon branches, illegal index `{}`" , idx ,) , }) . reduce (| | (None , None ,) , | (reserved_grust_rayon_opt_var_0 , reserved_grust_rayon_opt_var_1) , (reserved_grust_rayon_opt_var_0_rgt , reserved_grust_rayon_opt_var_1_rgt) | (match (reserved_grust_rayon_opt_var_0 , reserved_grust_rayon_opt_var_0_rgt) { (None , None) => None , (Some (val) , None) | (None , Some (val)) => Some (val) , (Some (_) , Some (_)) => unreachable ! ("fatal error in rayon reduce operation, found two values") } , match (reserved_grust_rayon_opt_var_1 , reserved_grust_rayon_opt_var_1_rgt) { (None , None) => None , (Some (val) , None) | (None , Some (val)) => Some (val) , (Some (_) , Some (_)) => unreachable ! ("fatal error in rayon reduce operation, found two values") }))
                };
                (
                    reserved_grust_rayon_opt_var_0.expect(
                        "unreachable: fatal error in final rayon unwrap, unexpected `None` value",
                    ),
                    reserved_grust_rayon_opt_var_1.expect(
                        "unreachable: fatal error in final rayon unwrap, unexpected `None` value",
                    ),
                )
            };
            let () = ();
            ((i12, i23), ())
        });
        let i123 = (i12 + (2i64 * i3)) + i23;
        let next_o = match input.i {
            0 => {
                let next_o = 1i64 + self.last_i;
                next_o
            }
            7 => {
                let next_o = i123;
                next_o
            }
            _ => {
                let next_o = i12;
                next_o
            }
        };
        self.last_i = input.i;
        next_o
    }
}
pub struct TestRayon1Input {
    pub i: i64,
}
pub struct TestRayon1State {
    last_i: i64,
    test_rayon1_aux: TestRayon1AuxState,
    test_rayon1_aux_1: TestRayon1AuxState,
    test_rayon1_aux_2: TestRayon1AuxState,
}
impl TestRayon1State {
    pub fn init() -> TestRayon1State {
        TestRayon1State {
            last_i: 0i64,
            test_rayon1_aux: TestRayon1AuxState::init(),
            test_rayon1_aux_1: TestRayon1AuxState::init(),
            test_rayon1_aux_2: TestRayon1AuxState::init(),
        }
    }
    pub fn step(&mut self, input: TestRayon1Input) -> i64 {
        let (i1_3, (i1_1, i1_2), ()) = std::thread::scope(|reserved_grust_thread_scope| {
            let i1_3 = {
                {
                    self.test_rayon1_aux.step(TestRayon1AuxInput { i: input.i })
                }
            };
            let (i1_1, i1_2) = {
                let (reserved_grust_rayon_opt_var_0, reserved_grust_rayon_opt_var_1) = {
                    #[allow(unused_imports)]
                    use grust::rayon::prelude::*;
                    (0 .. 2usize) . into_par_iter () . map (| idx : usize | match idx { 0usize => (Some ({ (input . i - 54i64) * 2i64 }) , None) , 1usize => (None , Some ({ (input . i + 54i64) * 2i64 })) , idx => unreachable ! ("fatal error in rayon branches, illegal index `{}`" , idx ,) , }) . reduce (| | (None , None ,) , | (reserved_grust_rayon_opt_var_0 , reserved_grust_rayon_opt_var_1) , (reserved_grust_rayon_opt_var_0_rgt , reserved_grust_rayon_opt_var_1_rgt) | (match (reserved_grust_rayon_opt_var_0 , reserved_grust_rayon_opt_var_0_rgt) { (None , None) => None , (Some (val) , None) | (None , Some (val)) => Some (val) , (Some (_) , Some (_)) => unreachable ! ("fatal error in rayon reduce operation, found two values") } , match (reserved_grust_rayon_opt_var_1 , reserved_grust_rayon_opt_var_1_rgt) { (None , None) => None , (Some (val) , None) | (None , Some (val)) => Some (val) , (Some (_) , Some (_)) => unreachable ! ("fatal error in rayon reduce operation, found two values") }))
                };
                (
                    reserved_grust_rayon_opt_var_0.expect(
                        "unreachable: fatal error in final rayon unwrap, unexpected `None` value",
                    ),
                    reserved_grust_rayon_opt_var_1.expect(
                        "unreachable: fatal error in final rayon unwrap, unexpected `None` value",
                    ),
                )
            };
            let () = ();
            (i1_3, (i1_1, i1_2), ())
        });
        let (((x, i2_1), (x_1, i2_2)), ()) = std::thread::scope(|reserved_grust_thread_scope| {
            let ((x, i2_1), (x_1, i2_2)) = (
                {
                    {
                        let x = (i1_1 + i1_2) - i1_3;
                        let i2_1 = self.test_rayon1_aux_1.step(TestRayon1AuxInput { i: x });
                        (x, i2_1)
                    }
                },
                {
                    {
                        let x_1 = (i1_2 - i1_2) + i1_3;
                        let i2_2 = self.test_rayon1_aux_2.step(TestRayon1AuxInput { i: x_1 });
                        (x_1, i2_2)
                    }
                },
            );
            let () = ();
            (((x, i2_1), (x_1, i2_2)), ())
        });
        let next_o = match input.i {
            0 => {
                let next_o = 1i64 + self.last_i;
                next_o
            }
            7 => {
                let next_o = i2_1;
                next_o
            }
            _ => {
                let next_o = i2_2;
                next_o
            }
        };
        self.last_i = input.i;
        next_o
    }
}
