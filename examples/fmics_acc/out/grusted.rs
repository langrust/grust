use creusot_contracts::{ensures, logic, open, prelude, requires, DeepModel};
#[derive(prelude :: Clone, Copy, prelude :: PartialEq, prelude :: Default, DeepModel)]
pub enum Activation {
    #[default]
    On,
    Off,
}
#[requires(0.0f64 <= sv_v && sv_v <= 50.0f64)]
#[requires(sv_v - fv_v <= 10.0f64 && fv_v <= 50.0f64)]
#[ensures(result <= 130.0f64)]
pub fn safety_distance(sv_v: f64, fv_v: f64) -> f64 {
    let rho = 1.0f64;
    let b_max = 0.6f64 * 9.81f64;
    let sv_d_stop = (sv_v * rho) + ((sv_v * sv_v) / (2.0f64 * b_max));
    let fv_d_stop = (fv_v * fv_v) / (2.0f64 * b_max);
    let d_safe = sv_d_stop - fv_d_stop;
    if d_safe < 0.0f64 {
        0.0f64
    } else {
        d_safe
    }
}
pub struct DeriveInput {
    pub d: f64,
    pub t_ms: f64,
}
pub struct DeriveState {
    last_d: f64,
    last_t_ms: f64,
    last_v_ms: f64,
    last_x: bool,
}
impl DeriveState {
    pub fn init() -> DeriveState {
        DeriveState {
            last_d: 0.0f64,
            last_t_ms: 0.0f64,
            last_v_ms: 0.0f64,
            last_x: false,
        }
    }
    #[requires(input.t_ms > self.last_t_ms)]
    pub fn step(&mut self, input: DeriveInput) -> f64 {
        let x = input.d != self.last_d;
        let v_ms = match () {
            () if x && !(self.last_x) => (input.d - self.last_d) / (input.t_ms - self.last_t_ms),
            () => self.last_v_ms,
        };
        let v_s = v_ms / 1000.0f64;
        self.last_d = input.d;
        self.last_t_ms = input.t_ms;
        self.last_v_ms = v_ms;
        self.last_x = x;
        v_s
    }
}
pub struct AccInput {
    pub c: bool,
    pub d: f64,
    pub v: f64,
    pub t: f64,
}
pub struct AccState {
    last_d: f64,
    last_d_dt: f64,
    last_fv_v: f64,
    last_t: f64,
    derive: DeriveState,
    derive_1: DeriveState,
    derive_2: DeriveState,
    derive_3: DeriveState,
    derive_4: DeriveState,
    derive_5: DeriveState,
}
impl AccState {
    pub fn init() -> AccState {
        AccState {
            last_d: 0.0f64,
            last_d_dt: 0.0f64,
            last_fv_v: 0.0f64,
            last_t: 0.0f64,
            derive: DeriveState::init(),
            derive_1: DeriveState::init(),
            derive_2: DeriveState::init(),
            derive_3: DeriveState::init(),
            derive_4: DeriveState::init(),
            derive_5: DeriveState::init(),
        }
    }
    #[requires(input.c == > 0.0f64 <= input.v && input.v <= 50.0f64)]
    #[requires(input.c == > -10.0f64 <=
    self.derive_1.step(DeriveInput
    { d : input.d, t_ms : input.t / 1000.0f64 }) &&
    self.derive_2.step(DeriveInput
    { d : input.d, t_ms : input.t / 1000.0f64 }) <= 0.0f64)]
    #[requires(input.c == > input.d >= input.v * 1.0f64 - input.v *
    self.derive_3.step(DeriveInput
    { d : input.d, t_ms : input.t / 1000.0f64 }) / 0.6f64 * 9.81f64)]
    #[ensures(input.c == > forall < _t : f64 > 0.0f64 < _t && _t <= -
    self.derive_4.step(DeriveInput
    { d : input.d, t_ms : input.t / 1000.0f64 }) / result == > input.d +
    self.derive_5.step(DeriveInput
    { d : input.d, t_ms : input.t / 1000.0f64 }) * _t + result * _t * _t /
    2.0f64 >= d_s)]
    #[ensures(0.0f64 <= result && result <= 0.6f64 * 9.81f64)]
    pub fn step(&mut self, input: AccInput) -> f64 {
        let (fv_v, b, d_dt, d_s) = match input.c {
            true => {
                let x = input.t / 1000.0f64;
                let d_dt = self.derive.step(DeriveInput {
                    d: input.d,
                    t_ms: x,
                });
                let fv_v = input.v + d_dt;
                let d_s = safety_distance(input.v, fv_v);
                let b = (d_dt * d_dt) / (input.d - d_s);
                (fv_v, b, d_dt, d_s)
            }
            false => {
                let d_dt = self.last_d_dt;
                let d_s = 0.0f64;
                let b = 0.0f64;
                let fv_v = self.last_fv_v;
                (fv_v, b, d_dt, d_s)
            }
        };
        self.last_d = input.d;
        self.last_d_dt = d_dt;
        self.last_fv_v = fv_v;
        self.last_t = input.t;
        b
    }
}
pub struct ActivateInput {
    pub act: Option<Activation>,
    pub r: Option<f64>,
}
pub struct ActivateState {
    last_active: bool,
    last_approach: bool,
    last_d: f64,
}
impl ActivateState {
    pub fn init() -> ActivateState {
        ActivateState {
            last_active: false,
            last_approach: false,
            last_d: 0.0f64,
        }
    }
    pub fn step(&mut self, input: ActivateInput) -> bool {
        let (active, d, approach) = match (input.act, input.r) {
            (Some(act), _) => {
                let active = act == Activation::On;
                (active, self.last_d, self.last_approach)
            }
            (_, Some(r)) => {
                let d = r;
                let approach = d < self.last_d;
                (self.last_active, d, approach)
            }
            (_, _) => (self.last_active, self.last_d, self.last_approach),
        };
        let c = active && approach;
        self.last_active = active;
        self.last_approach = approach;
        self.last_d = d;
        c
    }
}
pub struct ConvertInput {
    pub speed_km_h: f64,
}
pub struct ConvertState {}
impl ConvertState {
    pub fn init() -> ConvertState {
        ConvertState {}
    }
    pub fn step(&mut self, input: ConvertInput) -> f64 {
        let m_s = input.speed_km_h / 3.6f64;
        m_s
    }
}
