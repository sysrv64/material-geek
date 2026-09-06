use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShapeStyle {
    None,
    ExtraSmall,
    Small,
    Medium,
    Large,
    LargeIncreased,
    ExtraLarge,
    ExtraLargeIncreased,
    ExtraExtraLarge,
    Full,
}

impl ShapeStyle {
    pub const fn radius_dp(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::ExtraSmall => 4.0,
            Self::Small => 8.0,
            Self::Medium => 12.0,
            Self::Large => 16.0,
            Self::LargeIncreased => 20.0,
            Self::ExtraLarge => 28.0,
            Self::ExtraLargeIncreased => 32.0,
            Self::ExtraExtraLarge => 48.0,
            Self::Full => f32::INFINITY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expressive_radii_present() {
        assert_eq!(ShapeStyle::LargeIncreased.radius_dp(), 20.0);
        assert_eq!(ShapeStyle::ExtraLargeIncreased.radius_dp(), 32.0);
        assert_eq!(ShapeStyle::ExtraExtraLarge.radius_dp(), 48.0);
    }
}
