compiler_top::prelude! {}

#[test]
fn should_compile_this_quickly() {
    let top: ir0::Top = parse_quote! {
        #![dump = "tests/macro_outputs/sensor_fusion.rs", stats_depth = 10]

        use function module::invsqrt(x : float) -> float;
        use function module::asinf(x : float) -> float;
        use function module::atan2f(y : float, x : float) -> float;

        const PI: float = 3.141592653589793238462;
        const g: float = 9.80665;

        component acc_z(ax: float, ay: float, az : float, gravx: float, gravy: float, gravz : float) -> (accz : float){
            accz = ax * gravx + ay * gravy + az * gravz;
        }

        component acc_z_without_gravity(ax: float, ay: float, az : float, gravx: float, gravy: float, gravz : float) -> (acc_z : float){
            let acc_z_g: float  = acc_z(ax, ay, az, gravx, gravy, gravz);
            acc_z = acc_z_g - g;
        }

        component normalize_vec3(x: float, y: float, z: float) -> (nx: float, ny: float, nz: float) {
            let r: float = invsqrt(x * x + y * y + z * z);
            nx = r * x;
            ny = r * y;
            nz = r * z;
        }

        component normalize_quat(qw: float, qx: float, qy: float, qz: float) -> (nqw: float, nqx: float, nqy: float, nqz: float) {
            let r: float = invsqrt(qw * qw + qx * qx + qy * qy + qz * qz);
            nqw = r * qw;
            nqx = r * qx;
            nqy = r * qy;
            nqz = r * qz;
        }

        component integral_feedback(x : float, halfx: float) -> (integralFB : float) {
            let twoKi: float = 2.0 * 0.001;
            let estimator_attitude_update_dt: float = 1.0 / 250.0;

            init integralFB: float = 0.0;
            integralFB = (last integralFB) + ((twoKi * estimator_attitude_update_dt) * halfx);
        }

        component sensfusion6_quat(gx: float, gy: float, gz: float, ax: float, ay: float, az: float) -> (qw: float, qx: float, qy: float, qz: float) {
            let twoKp: float = 2.0 * 0.4;
            let estimator_attitude_update_dt: float = 1.0 / 250.0;

            init (qw, qx, qy, qz) = (1.0, 0.0, 0.0, 0.0);

            let grx: float = gx * PI / 180.0;
            let gry: float = gy * PI / 180.0;
            let grz: float = gz * PI / 180.0;

            let (arx: float, ary: float, arz: float) = normalize_vec3(ax, ay, az);

            let halfvx: float = last qx * last qz - last qw * last qy;
            let halfvy: float = last qw * last qx + last qy * last qz;
            let halfvz: float = last qw * last qw - 0.5 + last qz * last qz;

            let halfex: float = (ary * halfvz - arz * halfvy);
            let halfey: float = (arz * halfvx - arx * halfvz);
            let halfez: float = (arx * halfvy - ary * halfvx);

            match (! ((ax, ay, az) == (0.0, 0.0, 0.0))) {
                true => {
                    let gx1: float = grx + integral_feedback(grx, halfex) + (twoKp * halfex);
                    let gy1: float = gry + integral_feedback(gry, halfey) + (twoKp * halfey);
                    let gz1: float = grz + integral_feedback(grz, halfez) + (twoKp * halfez);
                },
                false => {
                    let gx1: float = grx;
                    let gy1: float = gry;
                    let gz1: float = grz;
                },
            }

            let gx2: float = gx1 * (0.5 * estimator_attitude_update_dt);
            let gy2: float = gy1 * (0.5 * estimator_attitude_update_dt);
            let gz2: float = gz1 * (0.5 * estimator_attitude_update_dt);

            let qwl: float = last qw - last qx * gx2 - last qy * gy2 - last qz * gz2;
            let qxl: float = last qx + last qw * gx2 + last qy * gz2 - last qz * gy2;
            let qyl: float = last qy + last qw * gy2 - last qx * gz2 + last qz * gx2;
            let qzl: float = last qz + last qw * gz2 + last qx * gy2 - last qy * gx2;

            let (nq1: float, nq2: float, nq3: float, nq4: float) = normalize_quat(qwl, qxl, qyl, qzl);

            (qw, qx, qy, qz) = (0.0, nq2, nq3, nq4);
        }
    };
    let (ast, mut ctx) = top.init();
    let tokens = compiler_top::into_token_stream(ast, &mut ctx);
    if let Some(path) = ctx.conf.dump_code {
        compiler_top::dump_code(&path, &tokens).unwrap();
    }
}
