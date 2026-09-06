use crate::spring::{spring_settled, spring_value};
use mg_tokens::motion_tokens::SpringSpec;

#[derive(Clone, Copy, Debug)]
pub struct DynamicSpring {
    pub x: f32,
    pub v: f32,
    pub target: f32,
    pub spec: SpringSpec,
    pub disp_thresh: f32,
    pub vel_thresh: f32,
}

impl DynamicSpring {
    pub fn new(x: f32, target: f32, spec: SpringSpec) -> Self {
        Self {
            x,
            v: 0.0,
            target,
            spec,
            disp_thresh: 0.01,
            vel_thresh: 0.01,
        }
    }

    pub fn retarget(&mut self, target: f32) {
        self.target = target;
    }

    pub fn retarget_with_velocity(&mut self, target: f32, v: f32) {
        self.target = target;
        self.v = v;
    }

    pub fn step(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 1.0 / 15.0);
        let (x, v) = spring_value(
            self.x,
            self.v,
            self.target,
            self.spec.damping_ratio,
            self.spec.stiffness,
            dt,
        );
        self.x = x;
        self.v = v;
        if self.settled() {
            self.x = self.target;
            self.v = 0.0;
        }
    }

    pub fn settled(&self) -> bool {
        spring_settled(
            self.x,
            self.v,
            self.target,
            self.disp_thresh,
            self.vel_thresh,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mg_tokens::motion_tokens::schemes;

    #[test]
    fn retarget_mid_flight_no_jump() {
        let mut s = DynamicSpring::new(0.0, 1.0, schemes::EXPRESSIVE_DEFAULT_SPATIAL);
        for _ in 0..30 {
            s.step(1.0 / 120.0);
        }
        let before = s.x;
        s.retarget(0.0);
        s.step(1.0 / 120.0);
        assert!((s.x - before).abs() < 0.2, "jump: {before} -> {}", s.x);
        for _ in 0..600 {
            s.step(1.0 / 120.0);
        }
        assert!(s.settled());
        assert!((s.x).abs() < 1e-6);
    }
}
