use mg_tokens::motion_tokens::{schemes, SpringSpec};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchemeKind {
    Expressive,
    Standard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tempo {
    Fast,
    Default,
    Slow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Track {
    Spatial,
    Effects,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MotionScheme {
    pub kind: SchemeKind,
}

impl MotionScheme {
    pub const fn expressive() -> Self {
        Self {
            kind: SchemeKind::Expressive,
        }
    }
    pub const fn standard() -> Self {
        Self {
            kind: SchemeKind::Standard,
        }
    }

    pub const fn spec(self, tempo: Tempo, track: Track) -> SpringSpec {
        use SchemeKind as K;
        use Tempo as T;
        use Track as R;
        match (self.kind, tempo, track) {
            (K::Expressive, T::Fast, R::Spatial) => schemes::EXPRESSIVE_FAST_SPATIAL,
            (K::Expressive, T::Default, R::Spatial) => schemes::EXPRESSIVE_DEFAULT_SPATIAL,
            (K::Expressive, T::Slow, R::Spatial) => schemes::EXPRESSIVE_SLOW_SPATIAL,
            (K::Expressive, T::Fast, R::Effects) => schemes::EXPRESSIVE_FAST_EFFECTS,
            (K::Expressive, T::Default, R::Effects) => schemes::EXPRESSIVE_DEFAULT_EFFECTS,
            (K::Expressive, T::Slow, R::Effects) => schemes::EXPRESSIVE_SLOW_EFFECTS,
            (K::Standard, T::Fast, R::Spatial) => schemes::STANDARD_FAST_SPATIAL,
            (K::Standard, T::Default, R::Spatial) => schemes::STANDARD_DEFAULT_SPATIAL,
            (K::Standard, T::Slow, R::Spatial) => schemes::STANDARD_SLOW_SPATIAL,
            (K::Standard, T::Fast, R::Effects) => schemes::STANDARD_FAST_EFFECTS,
            (K::Standard, T::Default, R::Effects) => schemes::STANDARD_DEFAULT_EFFECTS,
            (K::Standard, T::Slow, R::Effects) => schemes::STANDARD_SLOW_EFFECTS,
        }
    }

    pub fn default_component_specs(self) -> (SpringSpec, SpringSpec) {
        (
            self.spec(Tempo::Fast, Track::Spatial),
            self.spec(Tempo::Fast, Track::Effects),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn component_defaults_match_tokens() {
        let (s, e) = MotionScheme::expressive().default_component_specs();
        assert_eq!(s, schemes::EXPRESSIVE_FAST_SPATIAL);
        assert_eq!(e, schemes::EXPRESSIVE_FAST_EFFECTS);
    }
}
