pub fn cubic_bezier_y(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let cx = 3.0 * x1;
    let bx = 3.0 * (x2 - x1) - cx;
    let ax = 1.0 - cx - bx;
    let cy = 3.0 * y1;
    let by = 3.0 * (y2 - y1) - cy;
    let ay = 1.0 - cy - by;
    let sample_x = |t: f32| ((ax * t + bx) * t + cx) * t;
    let sample_dx = |t: f32| (3.0 * ax * t + 2.0 * bx) * t + cx;

    let mut t = x;
    for _ in 0..8 {
        let err = sample_x(t) - x;
        if err.abs() < 1e-6 {
            break;
        }
        let d = sample_dx(t);
        if d.abs() < 1e-6 {
            break;
        }
        t = (t - err / d).clamp(0.0, 1.0);
    }
    if (sample_x(t) - x).abs() > 1e-4 {
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        t = x;
        for _ in 0..24 {
            t = 0.5 * (lo + hi);
            if sample_x(t) < x {
                lo = t;
            } else {
                hi = t;
            }
        }
    }
    ((ay * t + by) * t + cy) * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use mg_tokens::motion_tokens::Easing;

    #[test]
    fn endpoints_exact() {
        assert_eq!(cubic_bezier_y(0.0, 0.2, 0.0, 0.0, 1.0), 0.0);
        assert_eq!(cubic_bezier_y(1.0, 0.2, 0.0, 0.0, 1.0), 1.0);
    }

    #[test]
    fn emphasized_monotonic_and_midpoint() {
        let e = Easing::EMPHASIZED;
        let mut prev = 0.0;
        for i in 1..=10 {
            let y = cubic_bezier_y(i as f32 / 10.0, e.x1, e.y1, e.x2, e.y2);
            assert!(y >= prev - 1e-5, "non-monotonic at {i}/10");
            prev = y;
        }
        assert!((prev - 1.0).abs() < 0.05);
    }

    #[test]
    fn linear_bezier_is_identity() {
        for i in 0..=10 {
            let x = i as f32 / 10.0;
            let y = cubic_bezier_y(x, 0.0, 0.0, 1.0, 1.0);
            assert!((y - x).abs() < 1e-4, "x={x} y={y}");
        }
    }
}
