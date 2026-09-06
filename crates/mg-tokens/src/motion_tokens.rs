use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Easing {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Easing {
    pub const EMPHASIZED: Self = Self {
        x1: 0.2,
        y1: 0.0,
        x2: 0.0,
        y2: 1.0,
    };
    pub const EMPHASIZED_DECELERATE: Self = Self {
        x1: 0.05,
        y1: 0.7,
        x2: 0.1,
        y2: 1.0,
    };
    pub const EMPHASIZED_ACCELERATE: Self = Self {
        x1: 0.3,
        y1: 0.0,
        x2: 0.8,
        y2: 0.15,
    };
    pub const STANDARD: Self = Self {
        x1: 0.2,
        y1: 0.0,
        x2: 0.0,
        y2: 1.0,
    };
    pub const STANDARD_DECELERATE: Self = Self {
        x1: 0.0,
        y1: 0.0,
        x2: 0.0,
        y2: 1.0,
    };
    pub const STANDARD_ACCELERATE: Self = Self {
        x1: 0.3,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DurationClass {
    Short1,
    Short2,
    Short3,
    Short4,
    Medium1,
    Medium2,
    Medium3,
    Medium4,
    Long1,
    Long2,
    Long3,
    Long4,
    ExtraLong1,
    ExtraLong2,
    ExtraLong3,
    ExtraLong4,
}

impl DurationClass {
    pub const fn ms(self) -> u16 {
        match self {
            Self::Short1 => 50,
            Self::Short2 => 100,
            Self::Short3 => 150,
            Self::Short4 => 200,
            Self::Medium1 => 250,
            Self::Medium2 => 300,
            Self::Medium3 => 350,
            Self::Medium4 => 400,
            Self::Long1 => 450,
            Self::Long2 => 500,
            Self::Long3 => 550,
            Self::Long4 => 600,
            Self::ExtraLong1 => 700,
            Self::ExtraLong2 => 800,
            Self::ExtraLong3 => 900,
            Self::ExtraLong4 => 1000,
        }
    }
}

pub struct MotionDurationsMs;

impl MotionDurationsMs {
    pub const EMPHASIZED_ON_SCREEN: u16 = 500;
    pub const EMPHASIZED_ENTER: u16 = 400;
    pub const EMPHASIZED_EXIT: u16 = 200;
    pub const STANDARD: u16 = 300;
    pub const STANDARD_DECELERATE: u16 = 250;
    pub const STANDARD_ACCELERATE: u16 = 200;
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpringSpec {
    pub damping_ratio: f32,
    pub stiffness: f32,
}

impl SpringSpec {
    pub const fn new(damping_ratio: f32, stiffness: f32) -> Self {
        Self {
            damping_ratio,
            stiffness,
        }
    }
    pub fn omega0(self) -> f32 {
        self.stiffness.sqrt()
    }
    pub fn critical_damping(self) -> f32 {
        2.0 * self.stiffness.sqrt()
    }
}

pub mod schemes {
    use super::SpringSpec;
    pub const EXPRESSIVE_FAST_SPATIAL: SpringSpec = SpringSpec::new(0.6, 800.0);
    pub const EXPRESSIVE_DEFAULT_SPATIAL: SpringSpec = SpringSpec::new(0.8, 380.0);
    pub const EXPRESSIVE_SLOW_SPATIAL: SpringSpec = SpringSpec::new(0.8, 200.0);
    pub const EXPRESSIVE_FAST_EFFECTS: SpringSpec = SpringSpec::new(1.0, 3800.0);
    pub const EXPRESSIVE_DEFAULT_EFFECTS: SpringSpec = SpringSpec::new(1.0, 1600.0);
    pub const EXPRESSIVE_SLOW_EFFECTS: SpringSpec = SpringSpec::new(1.0, 800.0);
    pub const STANDARD_FAST_SPATIAL: SpringSpec = SpringSpec::new(0.9, 1400.0);
    pub const STANDARD_DEFAULT_SPATIAL: SpringSpec = SpringSpec::new(0.9, 700.0);
    pub const STANDARD_SLOW_SPATIAL: SpringSpec = SpringSpec::new(0.9, 300.0);
    pub const STANDARD_FAST_EFFECTS: SpringSpec = SpringSpec::new(1.0, 3800.0);
    pub const STANDARD_DEFAULT_EFFECTS: SpringSpec = SpringSpec::new(1.0, 1600.0);
    pub const STANDARD_SLOW_EFFECTS: SpringSpec = SpringSpec::new(1.0, 800.0);
    pub const SPRING_DEFAULT: SpringSpec = SpringSpec::new(1.0, 1500.0);
    pub const SPRING_ENTER_EXIT: SpringSpec = SpringSpec::new(1.0, 400.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn effects_have_no_overshoot() {
        for s in [
            schemes::EXPRESSIVE_FAST_EFFECTS,
            schemes::EXPRESSIVE_DEFAULT_EFFECTS,
            schemes::EXPRESSIVE_SLOW_EFFECTS,
            schemes::STANDARD_FAST_EFFECTS,
        ] {
            assert_eq!(s.damping_ratio, 1.0);
        }
    }
    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn spatial_expressive_bouncier_than_standard() {
        assert!(
            schemes::EXPRESSIVE_DEFAULT_SPATIAL.damping_ratio
                < schemes::STANDARD_DEFAULT_SPATIAL.damping_ratio
        );
    }
}
