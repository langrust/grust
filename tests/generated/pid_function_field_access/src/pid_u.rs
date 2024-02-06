use crate::derive_d::*;
use crate::integrate_i::*;
use crate::typedefs::GainPID;
use crate::functions::access_k_p;
use crate::functions::access_k_i;
use crate::functions::access_k_d;
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
    pub fn step(&mut self, input: PidUInput) -> f64 {
        let e = input.v_c - input.v;
        let e_d = self.derive_d_e_d.step(DeriveDInput { x: e, dt: input.dt });
        let e_i = self
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
        let u = Expr::FunctionCall(
            parse_quote! {
                access_k_p(gain)
            },
        ) * e
            + Expr::FunctionCall(
                parse_quote! {
                    access_k_i(gain)
                },
            ) * e_i
            + Expr::FunctionCall(
                parse_quote! {
                    access_k_d(gain)
                },
            ) * e_d;
        u
    }
}
