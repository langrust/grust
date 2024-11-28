pub struct Test3AuxInput {
    pub i: i64,
}
pub struct Test3AuxState {
    last_i: i64,
}
impl Test3AuxState {
    pub fn init() -> Test3AuxState {
        Test3AuxState { last_i: 0i64 }
    }
    pub fn step(&mut self, input: Test3AuxInput) -> i64 {
        let (i1, i3, i2) = {
            let (i1, i3, i2) = {
                let (
                    reserved_grust_rayon_opt_var_0,
                    reserved_grust_rayon_opt_var_1,
                    reserved_grust_rayon_opt_var_2,
                ) = {
                    #[allow(unused_imports)]
                    use grust::rayon::prelude::*;
                    (0..3usize)
                        .into_par_iter()
                        .map(|idx: usize| match idx {
                            0usize => (Some((input.i - 54i64) * 2i64), None, None),
                            1usize => (None, Some(7i64 * input.i), None),
                            2usize => (None, None, Some((input.i + 54i64) * 2i64)),
                            idx => unreachable!(
                                "fatal error in rayon branches, illegal index `{}`",
                                idx,
                            ),
                        })
                        .reduce(
                            || (None, None, None),
                            |(
                                reserved_grust_rayon_opt_var_0,
                                reserved_grust_rayon_opt_var_1,
                                reserved_grust_rayon_opt_var_2,
                            ),
                             (
                                reserved_grust_rayon_opt_var_0_rgt,
                                reserved_grust_rayon_opt_var_1_rgt,
                                reserved_grust_rayon_opt_var_2_rgt,
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
                                    match (
                                        reserved_grust_rayon_opt_var_2,
                                        reserved_grust_rayon_opt_var_2_rgt,
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
            (i1, i3, i2)
        };
        let (i12, i23) = {
            let (i12, i23) = {
                let (reserved_grust_rayon_opt_var_0, reserved_grust_rayon_opt_var_1) = {
                    #[allow(unused_imports)]
                    use grust::rayon::prelude::*;
                    (0..2usize)
                        .into_par_iter()
                        .map(|idx: usize| match idx {
                            0usize => (Some(i1 + i2), None),
                            1usize => (None, Some(i2 + i3)),
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
                    reserved_grust_rayon_opt_var_0.expect(
                        "unreachable: fatal error in final rayon unwrap, unexpected `None` value",
                    ),
                    reserved_grust_rayon_opt_var_1.expect(
                        "unreachable: fatal error in final rayon unwrap, unexpected `None` value",
                    ),
                )
            };
            (i12, i23)
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
pub struct Test3Input {
    pub i: i64,
}
pub struct Test3State {
    last_i: i64,
    test3_aux: Test3AuxState,
    test3_aux_1: Test3AuxState,
    test3_aux_2: Test3AuxState,
}
impl Test3State {
    pub fn init() -> Test3State {
        Test3State {
            last_i: 0i64,
            test3_aux: Test3AuxState::init(),
            test3_aux_1: Test3AuxState::init(),
            test3_aux_2: Test3AuxState::init(),
        }
    }
    pub fn step(&mut self, input: Test3Input) -> i64 {
        let ((i1_1, i1_2), i1_3) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope
                .spawn(|| self.test3_aux.step(Test3AuxInput { i: input.i }));
            let (i1_1, i1_2) = {
                let (reserved_grust_rayon_opt_var_0, reserved_grust_rayon_opt_var_1) = {
                    #[allow(unused_imports)]
                    use grust::rayon::prelude::*;
                    (0..2usize)
                        .into_par_iter()
                        .map(|idx: usize| match idx {
                            0usize => (Some((input.i - 54i64) * 2i64), None),
                            1usize => (None, Some((input.i + 54i64) * 2i64)),
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
                    reserved_grust_rayon_opt_var_0.expect(
                        "unreachable: fatal error in final rayon unwrap, unexpected `None` value",
                    ),
                    reserved_grust_rayon_opt_var_1.expect(
                        "unreachable: fatal error in final rayon unwrap, unexpected `None` value",
                    ),
                )
            };
            let i1_3 = (reserved_grust_thread_kid_0
                .join()
                .expect("unexpected panic in sub-thread"));
            ((i1_1, i1_2), i1_3)
        });
        let ((x, i2_1), (x_1, i2_2)) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope.spawn(|| {
                let x = (i1_1 + i1_2) - i1_3;
                let i2_1 = self.test3_aux_1.step(Test3AuxInput { i: x });
                (x, i2_1)
            });
            let reserved_grust_thread_kid_1 = reserved_grust_thread_scope.spawn(|| {
                let x_1 = (i1_2 - i1_2) + i1_3;
                let i2_2 = self.test3_aux_2.step(Test3AuxInput { i: x_1 });
                (x_1, i2_2)
            });
            let ((x, i2_1), (x_1, i2_2)) = (
                reserved_grust_thread_kid_0
                    .join()
                    .expect("unexpected panic in sub-thread"),
                reserved_grust_thread_kid_1
                    .join()
                    .expect("unexpected panic in sub-thread"),
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
