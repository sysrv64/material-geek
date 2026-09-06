pub fn spring_value(
    x0: f32,
    v0: f32,
    target: f32,
    damping_ratio: f32,
    stiffness: f32,
    t: f32,
) -> (f32, f32) {
    debug_assert!(stiffness > 0.0 && t >= 0.0);
    let w0 = stiffness.sqrt();
    let z = damping_ratio;
    let d0 = x0 - target;

    if (z - 1.0).abs() < 1e-4 {
        let c1 = d0;
        let c2 = v0 + w0 * d0;
        let e = (-w0 * t).exp();
        let d = e * (c1 + c2 * t);
        let v = e * (c2 - w0 * (c1 + c2 * t));
        (target + d, v)
    } else if z < 1.0 {
        let wd = w0 * (1.0 - z * z).sqrt();
        let c1 = d0;
        let c2 = (v0 + z * w0 * d0) / wd;
        let e = (-z * w0 * t).exp();
        let (s, c) = (wd * t).sin_cos();
        let d = e * (c1 * c + c2 * s);
        let v = e * ((c2 * wd - z * w0 * c1) * c - (c1 * wd + z * w0 * c2) * s);
        (target + d, v)
    } else {
        let s = (z * z - 1.0).sqrt();
        let r1 = -w0 * (z - s);
        let r2 = -w0 * (z + s);
        let c2 = (v0 - r1 * d0) / (r2 - r1);
        let c1 = d0 - c2;
        let e1 = (r1 * t).exp();
        let e2 = (r2 * t).exp();
        let d = c1 * e1 + c2 * e2;
        let v = c1 * r1 * e1 + c2 * r2 * e2;
        (target + d, v)
    }
}

pub fn spring_settled(x: f32, v: f32, target: f32, disp_thresh: f32, vel_thresh: f32) -> bool {
    (x - target).abs() <= disp_thresh && v.abs() <= vel_thresh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critically_damped_converges_without_overshoot() {
        let (mut x, mut v) = (0.0f32, 0.0f32);
        let mut min = f32::INFINITY;
        let mut t = 0.0;
        while t < 2.0 {
            (x, v) = spring_value(x, v, 1.0, 1.0, 1500.0, 1.0 / 240.0);
            min = min.min(x);
            t += 1.0 / 240.0;
        }
        assert!((x - 1.0).abs() < 0.005, "x={x}");
        assert!(min >= -0.001, "overshoot at zeta=1, min={min}");
    }

    #[test]
    fn expressive_spatial_overshoots_effects_does_not() {
        let step = 1.0 / 240.0;
        let run = |z: f32, k: f32| {
            let (mut x, mut v) = (0.0f32, 0.0f32);
            let mut max = f32::NEG_INFINITY;
            for _ in 0..(2.0 / step) as usize {
                (x, v) = spring_value(x, v, 1.0, z, k, step);
                max = max.max(x);
            }
            (x, max)
        };
        let (xe, maxe) = run(0.8, 380.0);
        let (xf, maxf) = run(1.0, 1600.0);
        assert!((xe - 1.0).abs() < 0.01);
        assert!((xf - 1.0).abs() < 0.01);
        assert!(maxe > 1.001, "spatial must overshoot, max={maxe}");
        assert!(maxf <= 1.001, "effects must not overshoot, max={maxf}");
    }

    #[test]
    fn zero_time_is_identity() {
        let (x, v) = spring_value(0.3, 2.0, 1.0, 0.8, 380.0, 0.0);
        assert!((x - 0.3).abs() < 1e-6 && (v - 2.0).abs() < 1e-6);
    }
}
