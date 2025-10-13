pub struct AccZInput {
    pub ax: f64,
    pub ay: f64,
    pub az: f64,
    pub gravx: f64,
    pub gravy: f64,
    pub gravz: f64,
}
pub struct AccZOutput {
    pub accz: f64,
}
pub struct AccZState {}
impl grust::core::Component for AccZState {
    type Input = AccZInput;
    type Output = AccZOutput;
    fn init() -> AccZState {
        AccZState {}
    }
    fn step(&mut self, input: AccZInput) -> AccZOutput {
        let accz = ((input.ax * input.gravx) + (input.ay * input.gravy)) + (input.az * input.gravz);
        AccZOutput { accz }
    }
}
pub struct AccZWithoutGravityInput {
    pub ax: f64,
    pub ay: f64,
    pub az: f64,
    pub gravx: f64,
    pub gravy: f64,
    pub gravz: f64,
}
pub struct AccZWithoutGravityOutput {
    pub acc_z: f64,
}
pub struct AccZWithoutGravityState {
    acc_z_1: AccZState,
}
impl grust::core::Component for AccZWithoutGravityState {
    type Input = AccZWithoutGravityInput;
    type Output = AccZWithoutGravityOutput;
    fn init() -> AccZWithoutGravityState {
        AccZWithoutGravityState {
            acc_z_1: <AccZState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: AccZWithoutGravityInput) -> AccZWithoutGravityOutput {
        let acc_z_g = {
            let AccZOutput { accz } = <AccZState as grust::core::Component>::step(
                &mut self.acc_z_1,
                AccZInput {
                    ax: input.ax,
                    ay: input.ay,
                    az: input.az,
                    gravx: input.gravx,
                    gravy: input.gravy,
                    gravz: input.gravz,
                },
            );
            (accz)
        };
        let acc_z = acc_z_g - 9.80665f64;
        AccZWithoutGravityOutput { acc_z }
    }
}
pub struct NormalizeVec3Input {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
pub struct NormalizeVec3Output {
    pub nx: f64,
    pub ny: f64,
    pub nz: f64,
}
pub struct NormalizeVec3State {}
impl grust::core::Component for NormalizeVec3State {
    type Input = NormalizeVec3Input;
    type Output = NormalizeVec3Output;
    fn init() -> NormalizeVec3State {
        NormalizeVec3State {}
    }
    fn step(&mut self, input: NormalizeVec3Input) -> NormalizeVec3Output {
        let r = module::invsqrt(((input.x * input.x) + (input.y * input.y)) + (input.z * input.z));
        let nx = r * input.x;
        let ny = r * input.y;
        let nz = r * input.z;
        NormalizeVec3Output { nx, ny, nz }
    }
}
pub struct NormalizeQuatInput {
    pub qw: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
}
pub struct NormalizeQuatOutput {
    pub nqw: f64,
    pub nqx: f64,
    pub nqy: f64,
    pub nqz: f64,
}
pub struct NormalizeQuatState {}
impl grust::core::Component for NormalizeQuatState {
    type Input = NormalizeQuatInput;
    type Output = NormalizeQuatOutput;
    fn init() -> NormalizeQuatState {
        NormalizeQuatState {}
    }
    fn step(&mut self, input: NormalizeQuatInput) -> NormalizeQuatOutput {
        let r = module::invsqrt(
            (((input.qw * input.qw) + (input.qx * input.qx)) + (input.qy * input.qy))
                + (input.qz * input.qz),
        );
        let nqw = r * input.qw;
        let nqx = r * input.qx;
        let nqy = r * input.qy;
        let nqz = r * input.qz;
        NormalizeQuatOutput { nqw, nqx, nqy, nqz }
    }
}
pub struct IntegralFeedbackInput {
    pub halfx: f64,
}
pub struct IntegralFeedbackOutput {
    pub integralFB: f64,
}
pub struct IntegralFeedbackState {
    last_integral_f_b: f64,
}
impl grust::core::Component for IntegralFeedbackState {
    type Input = IntegralFeedbackInput;
    type Output = IntegralFeedbackOutput;
    fn init() -> IntegralFeedbackState {
        IntegralFeedbackState {
            last_integral_f_b: 0.0f64,
        }
    }
    fn step(&mut self, input: IntegralFeedbackInput) -> IntegralFeedbackOutput {
        let twoKi = 2.0f64 * 0.001f64;
        let estimator_attitude_update_dt = 1.0f64 / 250.0f64;
        let integralFB =
            self.last_integral_f_b + ((twoKi * estimator_attitude_update_dt) * input.halfx);
        self.last_integral_f_b = integralFB;
        IntegralFeedbackOutput { integralFB }
    }
}
pub struct Sensfusion6QuatInput {
    pub gx: f64,
    pub gy: f64,
    pub gz: f64,
    pub ax: f64,
    pub ay: f64,
    pub az: f64,
}
pub struct Sensfusion6QuatOutput {
    pub qw: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
}
pub struct Sensfusion6QuatState {
    last_qw: f64,
    last_qx: f64,
    last_qy: f64,
    last_qz: f64,
    normalize_vec3: NormalizeVec3State,
    integral_feedback: IntegralFeedbackState,
    integral_feedback_1: IntegralFeedbackState,
    integral_feedback_2: IntegralFeedbackState,
    normalize_quat: NormalizeQuatState,
}
impl grust::core::Component for Sensfusion6QuatState {
    type Input = Sensfusion6QuatInput;
    type Output = Sensfusion6QuatOutput;
    fn init() -> Sensfusion6QuatState {
        Sensfusion6QuatState {
            last_qw: 1.0f64,
            last_qx: 0.0f64,
            last_qy: 0.0f64,
            last_qz: 0.0f64,
            normalize_vec3: <NormalizeVec3State as grust::core::Component>::init(),
            integral_feedback: <IntegralFeedbackState as grust::core::Component>::init(),
            integral_feedback_1: <IntegralFeedbackState as grust::core::Component>::init(),
            integral_feedback_2: <IntegralFeedbackState as grust::core::Component>::init(),
            normalize_quat: <NormalizeQuatState as grust::core::Component>::init(),
        }
    }
    fn step(&mut self, input: Sensfusion6QuatInput) -> Sensfusion6QuatOutput {
        let twoKp = 2.0f64 * 0.4f64;
        let estimator_attitude_update_dt = 1.0f64 / 250.0f64;
        let grx = (input.gx * 3.141592653589793238462f64) / 180.0f64;
        let gry = (input.gy * 3.141592653589793238462f64) / 180.0f64;
        let grz = (input.gz * 3.141592653589793238462f64) / 180.0f64;
        let (arx, ary, arz) = {
            let NormalizeVec3Output { nx, ny, nz } =
                <NormalizeVec3State as grust::core::Component>::step(
                    &mut self.normalize_vec3,
                    NormalizeVec3Input {
                        x: input.ax,
                        y: input.ay,
                        z: input.az,
                    },
                );
            (nx, ny, nz)
        };
        let halfvx = (self.last_qx * self.last_qz) - (self.last_qw * self.last_qy);
        let halfvy = (self.last_qw * self.last_qx) + (self.last_qy * self.last_qz);
        let halfvz = ((self.last_qw * self.last_qw) - 0.5f64) + (self.last_qz * self.last_qz);
        let halfex = (ary * halfvz) - (arz * halfvy);
        let halfey = (arz * halfvx) - (arx * halfvz);
        let halfez = (arx * halfvy) - (ary * halfvx);
        let (gx1, gz1, gy1) = match !((input.ax, input.ay, input.az) == (0.0f64, 0.0f64, 0.0f64)) {
            true => {
                let comp_app_integral_feedback = {
                    let IntegralFeedbackOutput { integralFB } =
                        <IntegralFeedbackState as grust::core::Component>::step(
                            &mut self.integral_feedback,
                            IntegralFeedbackInput { halfx: halfex },
                        );
                    (integralFB)
                };
                let gx1 = (grx + comp_app_integral_feedback) + (twoKp * halfex);
                let comp_app_integral_feedback_1 = {
                    let IntegralFeedbackOutput { integralFB } =
                        <IntegralFeedbackState as grust::core::Component>::step(
                            &mut self.integral_feedback_1,
                            IntegralFeedbackInput { halfx: halfey },
                        );
                    (integralFB)
                };
                let gy1 = (gry + comp_app_integral_feedback_1) + (twoKp * halfey);
                let comp_app_integral_feedback_2 = {
                    let IntegralFeedbackOutput { integralFB } =
                        <IntegralFeedbackState as grust::core::Component>::step(
                            &mut self.integral_feedback_2,
                            IntegralFeedbackInput { halfx: halfez },
                        );
                    (integralFB)
                };
                let gz1 = (grz + comp_app_integral_feedback_2) + (twoKp * halfez);
                (gx1, gz1, gy1)
            }
            false => {
                let gx1 = grx;
                let gy1 = gry;
                let gz1 = grz;
                (gx1, gz1, gy1)
            }
        };
        let gx2 = gx1 * (0.5f64 * estimator_attitude_update_dt);
        let gy2 = gy1 * (0.5f64 * estimator_attitude_update_dt);
        let gz2 = gz1 * (0.5f64 * estimator_attitude_update_dt);
        let qwl =
            ((self.last_qw - (self.last_qx * gx2)) - (self.last_qy * gy2)) - (self.last_qz * gz2);
        let qxl =
            ((self.last_qx + (self.last_qw * gx2)) + (self.last_qy * gz2)) - (self.last_qz * gy2);
        let qyl =
            ((self.last_qy + (self.last_qw * gy2)) - (self.last_qx * gz2)) + (self.last_qz * gx2);
        let qzl =
            ((self.last_qz + (self.last_qw * gz2)) + (self.last_qx * gy2)) - (self.last_qy * gx2);
        let (qw, qx, qy, qz) = {
            let NormalizeQuatOutput { nqw, nqx, nqy, nqz } =
                <NormalizeQuatState as grust::core::Component>::step(
                    &mut self.normalize_quat,
                    NormalizeQuatInput {
                        qw: qwl,
                        qx: qxl,
                        qy: qyl,
                        qz: qzl,
                    },
                );
            (nqw, nqx, nqy, nqz)
        };
        self.last_qw = qw;
        self.last_qx = qx;
        self.last_qy = qy;
        self.last_qz = qz;
        Sensfusion6QuatOutput { qw, qx, qy, qz }
    }
}
