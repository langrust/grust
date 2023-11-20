use crate::derive_d::*;
use crate::integrate_i::*;
use crate::typedefs::GainPID;
pub struct PidUInput {
    pub v_c: f64,
    pub v: f64,
    pub dt: f64,
}
pub struct PidUState {
    derive_d_e_d: DeriveDState,
    integrate_i_e_i: IntegrateIState,
}
impl PidUState {
    pub fn init() -> PidUState {
        PidUState {
            derive_d_e_d: DeriveDState::init(),
            integrate_i_e_i: IntegrateIState::init(),
        }
    }
    pub fn step(self, input: PidUInput) -> (PidUState, f64) {
        let e = input.v_c - input.v;
        let (derive_d_e_d, e_d) = self
            .derive_d_e_d
            .step(DeriveDInput { x: e, dt: input.dt });
        let (integrate_i_e_i, e_i) = self
            .integrate_i_e_i
            .step(IntegrateIInput {
                x: e,
                dt: input.dt,
            });
        let gain = GainPID {
            k_p: 0.2f64,
            k_i: 1.5f64,
            k_d: 6f64,
        };
        let u = match gain {
            GainPID { k_p, k_i, k_d } => k_p * e + k_i * e_i + k_d * e_d,
        };
        (
            PidUState {
                derive_d_e_d,
                integrate_i_e_i,
            },
            u,
        )
    }
}
