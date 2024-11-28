pub struct Test2Input {
    pub i: i64,
}
pub struct Test2State {
    last_i: i64,
}
impl Test2State {
    pub fn init() -> Test2State {
        Test2State { last_i: 0i64 }
    }
    pub fn step(&mut self, input: Test2Input) -> i64 {
        let (i1, i3, i2) = {
            let (i1, i3, i2) = {
                let (
                    reserved_grust_rayon_opt_var_0,
                    reserved_grust_rayon_opt_var_1,
                    reserved_grust_rayon_opt_var_2,
                ) = {
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
