pub mod integration {
    /// Backward Euler method.
    ///
    /// Implements `F_n+1 = F_n + (t_n+1 - t_n) * f_n`.
    /// Called in `grust!` macro via
    /// `use component core::time::integration::backward_euler(x: float, t: float) -> (i: float);`.
    pub struct BackwardEulerState {
        integral: f64,
        last_t: f64,
        last_x: f64,
    }
    pub struct BackwardEulerInput {
        pub x: f64,
        pub t: f64,
    }
    impl grust_core::Component for BackwardEulerState {
        type Input = BackwardEulerInput;
        type Output = f64;

        fn init() -> Self {
            BackwardEulerState {
                integral: 0.,
                last_t: 0.,
                last_x: 0.,
            }
        }

        fn step(&mut self, input: Self::Input) -> Self::Output {
            let dt = input.t - self.last_t;
            let integral = self.integral + self.last_x * dt;
            self.integral = integral;
            self.last_t = input.t;
            self.last_x = input.x;
            integral
        }
    }

    #[cfg(test)]
    mod backward_euler {
        use grust_core::Component;

        use super::{BackwardEulerInput, BackwardEulerState};

        impl Clone for BackwardEulerInput {
            fn clone(&self) -> Self {
                Self {
                    x: self.x,
                    t: self.t,
                }
            }
        }

        #[test]
        fn should_be_resilient_to_oversampling() {
            let mut euler_1 = BackwardEulerState::init();
            let mut euler_2 = BackwardEulerState::init();
            let i1 = BackwardEulerInput { x: 4.0, t: 0.33 };
            let i_over_sample = BackwardEulerInput { x: 4.0, t: 1.77 };
            let i2 = BackwardEulerInput { x: 6.6, t: 2.13 };

            let _ = (euler_1.step(i1.clone()), euler_2.step(i1));
            let _ = euler_1.step(i_over_sample); // over sample
            let (o1, o2) = (euler_1.step(i2.clone()), euler_2.step(i2));

            assert_eq!(o1, o2)
        }
    }

    /// Trapeze method.
    ///
    /// Called in `grust!` macro via
    /// `use component core::time::integration::trapeze(x: float, t: float) -> (i: float);`.
    pub struct TrapezeState {
        integral: f64,
        last_t: f64,
        last_x: f64,
    }
    pub struct TrapezeInput {
        pub x: f64,
        pub t: f64,
    }
    impl grust_core::Component for TrapezeState {
        type Input = TrapezeInput;
        type Output = f64;

        fn init() -> Self {
            TrapezeState {
                integral: 0.,
                last_t: 0.,
                last_x: 0.,
            }
        }

        fn step(&mut self, input: Self::Input) -> Self::Output {
            let dt = input.t - self.last_t;
            let integral = self.integral + (input.x + self.last_x) * dt / 2.;
            if self.last_x != input.x {
                self.integral = integral;
                self.last_t = input.t;
                self.last_x = input.x;
            }
            integral
        }
    }

    #[cfg(test)]
    mod trapeze {
        use grust_core::Component;

        use super::{TrapezeInput, TrapezeState};

        impl Clone for TrapezeInput {
            fn clone(&self) -> Self {
                Self {
                    x: self.x,
                    t: self.t,
                }
            }
        }

        #[test]
        fn should_be_resilient_to_oversampling() {
            let mut trapeze_1 = TrapezeState::init();
            let mut trapeze_2 = TrapezeState::init();
            let i1 = TrapezeInput { x: 4.0, t: 0.33 };
            let i_over_sample = TrapezeInput { x: 4.0, t: 1.77 };
            let i2 = TrapezeInput { x: 6.6, t: 2.13 };

            let _ = (trapeze_1.step(i1.clone()), trapeze_2.step(i1));
            let _ = trapeze_1.step(i_over_sample); // over sample
            let (o1, o2) = (trapeze_1.step(i2.clone()), trapeze_2.step(i2));

            assert_eq!(o1, o2)
        }
    }

    /// Forward Euler method.
    ///
    /// F_n+1 = F_n + (t_n+1 - t_n) * f_n+1
    pub struct ForwardEulerState {
        integral: f64,
        last_t: f64,
    }
    pub struct ForwardEulerInput {
        x: f64,
        t: f64,
    }
    impl grust_core::Component for ForwardEulerState {
        type Input = ForwardEulerInput;
        type Output = f64;

        fn init() -> Self {
            ForwardEulerState {
                integral: 0.,
                last_t: 0.,
            }
        }

        fn step(&mut self, input: Self::Input) -> Self::Output {
            let dt = input.t - self.last_t;
            let integral = self.integral + input.x * dt;
            self.integral = integral;
            self.last_t = input.t;
            let _result = integral;
            panic!("not resilient to_oversampling");
        }
    }

    #[cfg(test)]
    mod forward_euler {
        use grust_core::Component;

        use super::{ForwardEulerInput, ForwardEulerState};

        impl Clone for ForwardEulerInput {
            fn clone(&self) -> Self {
                Self {
                    x: self.x,
                    t: self.t,
                }
            }
        }

        #[test]
        #[should_panic]
        fn not_resilient_to_oversampling() {
            let mut euler_1 = ForwardEulerState::init();
            let mut euler_2 = ForwardEulerState::init();
            let i1 = ForwardEulerInput { x: 4.0, t: 0.33 };
            let i_over_sample = ForwardEulerInput { x: 4.0, t: 1.77 };
            let i2 = ForwardEulerInput { x: 6.6, t: 2.13 };

            let _ = (euler_1.step(i1.clone()), euler_2.step(i1));
            let _ = euler_1.step(i_over_sample); // over sample
            let (o1, o2) = (euler_1.step(i2.clone()), euler_2.step(i2));

            assert_eq!(o1, o2)
        }
    }
}

pub mod derivation {
    /// Derivation method.
    ///
    /// Called in `grust!` macro via
    /// `use component core::time::derivation::derive(x: float, t: float) -> (d: float);`.
    pub struct DeriveState {
        last_t: f64,
        last_x: f64,
    }
    pub struct DeriveInput {
        pub x: f64,
        pub t: f64,
    }
    impl grust_core::Component for DeriveState {
        type Input = DeriveInput;
        type Output = f64;

        fn init() -> Self {
            DeriveState {
                last_t: 0.,
                last_x: 0.,
            }
        }

        fn step(&mut self, input: Self::Input) -> Self::Output {
            let dt = input.t - self.last_t;
            let dx = input.x - self.last_x;
            if self.last_x != input.x {
                self.last_t = input.t;
                self.last_x = input.x;
            }
            dx / dt
        }
    }

    #[cfg(test)]
    mod derive {
        use grust_core::Component;

        use super::{DeriveInput, DeriveState};

        impl Clone for DeriveInput {
            fn clone(&self) -> Self {
                Self {
                    x: self.x,
                    t: self.t,
                }
            }
        }

        #[test]
        fn should_be_resilient_to_oversampling() {
            let mut derive_1 = DeriveState::init();
            let mut derive_2 = DeriveState::init();
            let i1 = DeriveInput { x: 4.0, t: 0.33 };
            let i_over_sample = DeriveInput { x: 4.0, t: 1.77 };
            let i2 = DeriveInput { x: 6.6, t: 2.13 };

            let _ = (derive_1.step(i1.clone()), derive_2.step(i1));
            let _ = derive_1.step(i_over_sample); // over sample
            let (o1, o2) = (derive_1.step(i2.clone()), derive_2.step(i2));

            assert_eq!(o1, o2)
        }
    }
}
