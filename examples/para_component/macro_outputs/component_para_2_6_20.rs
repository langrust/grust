pub struct TestCustomAuxInput {
    pub i: i64,
}
pub struct TestCustomAuxState {
    last_i: i64,
}
impl TestCustomAuxState {
    pub fn init() -> TestCustomAuxState {
        TestCustomAuxState { last_i: 0i64 }
    }
    pub fn step(&mut self, input: TestCustomAuxInput) -> i64 {
        let (i3, (i1, i2), ()) = {
            let i3 = {
                {
                    7i64 * input.i
                }
            };
            let (i1, i2) = {
                let (reserved_grust_rayon_opt_var_0, reserved_grust_rayon_opt_var_1) = {
                    #[allow(unused_imports)]
                    use grust::rayon::prelude::*;
                    (0..2usize)
                        .into_par_iter()
                        .map(|idx: usize| match idx {
                            0usize => (Some({ (input.i - 54i64) * 2i64 }), None),
                            1usize => (None, Some({ (input.i + 54i64) * 2i64 })),
                            idx => unreachable!(
                                "fatal error in rayon branches, illegal index `{}`",
                                idx,
                            ),
                        })
                        .reduce(
                            || (None, None),
                            |(reserved_grust_rayon_opt_var_0, reserved_grust_rayon_opt_var_1),
                             (
                                reserved_grust_rayon_opt_var_0_rgt,
                                reserved_grust_rayon_opt_var_1_rgt,
                            )| {
                                (
                                    match (
                                        reserved_grust_rayon_opt_var_0,
                                        reserved_grust_rayon_opt_var_0_rgt,
                                    ) {
                                        (None, None) => None,
                                        (Some(val), None) | (None, Some(val)) => Some(val),
                                        (Some(_), Some(_)) => unreachable!
                        ("fatal error in rayon reduce operation, found two values"),
                                    },
                                    match (
                                        reserved_grust_rayon_opt_var_1,
                                        reserved_grust_rayon_opt_var_1_rgt,
                                    ) {
                                        (None, None) => None,
                                        (Some(val), None) | (None, Some(val)) => Some(val),
                                        (Some(_), Some(_)) => unreachable!
                        ("fatal error in rayon reduce operation, found two values"),
                                    },
                                )
                            },
                        )
                };
                (
                    {
                        reserved_grust_rayon_opt_var_0.expect("unreachable: fatal error in final rayon unwrap, unexpected `None` value")
                    },
                    {
                        reserved_grust_rayon_opt_var_1.expect("unreachable: fatal error in final rayon unwrap, unexpected `None` value")
                    },
                )
            };
            (i3, (i1, i2), ())
        };
        let ((i12, i23), ()) = {
            let (i12, i23) = (
                {
                    {
                        i1 + i2
                    }
                },
                {
                    {
                        i2 + i3
                    }
                },
            );
            ((i12, i23), ())
        };
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
pub struct TestCustomInput {
    pub i: i64,
}
pub struct TestCustomState {
    last_i: i64,
    test_custom_aux: TestCustomAuxState,
    test_custom_aux_1: TestCustomAuxState,
    test_custom_aux_2: TestCustomAuxState,
}
impl TestCustomState {
    pub fn init() -> TestCustomState {
        TestCustomState {
            last_i: 0i64,
            test_custom_aux: TestCustomAuxState::init(),
            test_custom_aux_1: TestCustomAuxState::init(),
            test_custom_aux_2: TestCustomAuxState::init(),
        }
    }
    pub fn step(&mut self, input: TestCustomInput) -> i64 {
        let ((i1_1, i1_2), i1_3) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope
                .spawn(|| self.test_custom_aux.step(TestCustomAuxInput { i: input.i }));
            let (i1_1, i1_2) = {
                let (reserved_grust_rayon_opt_var_0, reserved_grust_rayon_opt_var_1) = {
                    #[allow(unused_imports)]
                    use grust::rayon::prelude::*;
                    (0..2usize)
                        .into_par_iter()
                        .map(|idx: usize| match idx {
                            0usize => (Some({ (input.i - 54i64) * 2i64 }), None),
                            1usize => (None, Some({ (input.i + 54i64) * 2i64 })),
                            idx => unreachable!(
                                "fatal error in rayon branches, illegal index `{}`",
                                idx,
                            ),
                        })
                        .reduce(
                            || (None, None),
                            |(reserved_grust_rayon_opt_var_0, reserved_grust_rayon_opt_var_1),
                             (
                                reserved_grust_rayon_opt_var_0_rgt,
                                reserved_grust_rayon_opt_var_1_rgt,
                            )| {
                                (
                                    match (
                                        reserved_grust_rayon_opt_var_0,
                                        reserved_grust_rayon_opt_var_0_rgt,
                                    ) {
                                        (None, None) => None,
                                        (Some(val), None) | (None, Some(val)) => Some(val),
                                        (Some(_), Some(_)) => unreachable!
                        ("fatal error in rayon reduce operation, found two values"),
                                    },
                                    match (
                                        reserved_grust_rayon_opt_var_1,
                                        reserved_grust_rayon_opt_var_1_rgt,
                                    ) {
                                        (None, None) => None,
                                        (Some(val), None) | (None, Some(val)) => Some(val),
                                        (Some(_), Some(_)) => unreachable!
                        ("fatal error in rayon reduce operation, found two values"),
                                    },
                                )
                            },
                        )
                };
                (
                    {
                        reserved_grust_rayon_opt_var_0.expect("unreachable: fatal error in final rayon unwrap, unexpected `None` value")
                    },
                    {
                        reserved_grust_rayon_opt_var_1.expect("unreachable: fatal error in final rayon unwrap, unexpected `None` value")
                    },
                )
            };
            let i1_3 = {
                reserved_grust_thread_kid_0
                    .join()
                    .expect("unexpected panic in sub-thread")
            };
            ((i1_1, i1_2), i1_3)
        });
        let ((x, i2_1), (x_1, i2_2)) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope.spawn(|| {
                let x = (i1_1 + i1_2) - i1_3;
                let i2_1 = self.test_custom_aux_1.step(TestCustomAuxInput { i: x });
                (x, i2_1)
            });
            let reserved_grust_thread_kid_1 = reserved_grust_thread_scope.spawn(|| {
                let x_1 = (i1_2 - i1_2) + i1_3;
                let i2_2 = self.test_custom_aux_2.step(TestCustomAuxInput { i: x_1 });
                (x_1, i2_2)
            });
            let ((x, i2_1), (x_1, i2_2)) = (
                {
                    reserved_grust_thread_kid_0
                        .join()
                        .expect("unexpected panic in sub-thread")
                },
                {
                    reserved_grust_thread_kid_1
                        .join()
                        .expect("unexpected panic in sub-thread")
                },
            );
            ((x, i2_1), (x_1, i2_2))
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
