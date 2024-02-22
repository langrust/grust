use crate::derive_d::*;
use crate::integrate_i::*;
use crate::typedefs::GainPID;
pub struct PidUInput {
    pub v_c: f64,
    pub v: f64,
    pub dt: f64,
}
pub struct PidUState {
    integrate_i: IntegrateIState,
    derive_d: DeriveDState,
}
impl PidUState {
    pub fn init() -> PidUState {
        PidUState {
            integrate_i: IntegrateIState::init(),
            derive_d: DeriveDState::init(),
        }
    }
    pub fn step(&mut self, input: PidUInput) -> f64 {
        let e = input.v_c - input.v;
        let e_d = self.derive_d.step(DeriveDInput { x: e, dt: input.dt });
        let e_i = self
            .integrate_i
            .step(IntegrateIInput {
                x: e,
                dt: input.dt,
            });
        let gain = GainPID {
            k_p: 0.2f64,
            k_i: 1.5f64,
            k_d: 6f64,
        };
        let u = match (gain) {
            GainPID { k_p: k_p, k_i: k_i, k_d: k_d } => k_p * e + k_i * e_i + k_d * e_d,
        };
        u
    }
}
