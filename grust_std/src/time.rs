pub mod integration {
    /// Forward Euler method.
    ///
    /// F_n+1 = F_n + (t_n+1 - t_n) * f_n+1
    pub struct ForwardEulerState {
        integral: f64,
        last_t: f64,
    }
    pub struct ForwardEulerInput {
        pub x: f64,
        pub t: f64,
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
            integral
        }
    }

    /// Backward Euler method.
    ///
    /// F_n+1 = F_n + (t_n+1 - t_n) * f_n
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
}
