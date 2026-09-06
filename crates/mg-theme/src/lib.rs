use mg_motion::{MotionScheme, SchemeKind};
use mg_tokens::color::ColorScheme;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeekTheme {
    pub scheme: ColorScheme,
    pub motion: MotionScheme,
    pub expressive_motion: bool,
}

impl GeekTheme {
    pub fn light_expressive() -> Self {
        Self {
            scheme: ColorScheme::light(),
            motion: MotionScheme::expressive(),
            expressive_motion: true,
        }
    }
    pub fn dark_expressive() -> Self {
        Self {
            scheme: ColorScheme::dark(),
            motion: MotionScheme::expressive(),
            expressive_motion: true,
        }
    }
    pub fn light_standard() -> Self {
        Self {
            scheme: ColorScheme::light(),
            motion: MotionScheme::standard(),
            expressive_motion: false,
        }
    }
    pub fn dark_standard() -> Self {
        Self {
            scheme: ColorScheme::dark(),
            motion: MotionScheme::standard(),
            expressive_motion: false,
        }
    }

    pub fn scheme_kind(&self) -> SchemeKind {
        self.motion.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expressive_is_default_geek_choice() {
        let t = GeekTheme::dark_expressive();
        assert_eq!(t.scheme_kind(), SchemeKind::Expressive);
        assert!(t.scheme.is_dark);
    }
}
