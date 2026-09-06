use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElevationLevel {
    L0 = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
    L4 = 4,
    L5 = 5,
}

impl ElevationLevel {
    pub const fn dp(self) -> f32 {
        match self {
            Self::L0 => 0.0,
            Self::L1 => 1.0,
            Self::L2 => 3.0,
            Self::L3 => 6.0,
            Self::L4 => 8.0,
            Self::L5 => 12.0,
        }
    }
}
