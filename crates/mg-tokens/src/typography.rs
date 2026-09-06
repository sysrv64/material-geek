use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeRole {
    DisplayL,
    DisplayM,
    DisplayS,
    HeadlineL,
    HeadlineM,
    HeadlineS,
    TitleL,
    TitleM,
    TitleS,
    BodyL,
    BodyM,
    BodyS,
    LabelL,
    LabelM,
    LabelS,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TypeStyle {
    pub size_sp: f32,
    pub line_height_sp: f32,
    pub weight: u16,
    pub tracking_em: f32,
    pub emphasized: bool,
}

impl TypeStyle {
    pub const fn of(size_sp: f32, line_height_sp: f32, weight: u16, tracking_em: f32) -> Self {
        Self {
            size_sp,
            line_height_sp,
            weight,
            tracking_em,
            emphasized: false,
        }
    }
    pub const fn emphasized(self) -> Self {
        let w = if self.weight + 100 > 700 {
            700
        } else {
            self.weight + 100
        };
        Self {
            weight: w,
            emphasized: true,
            ..self
        }
    }
}

pub fn style(role: TypeRole, emphasized: bool) -> TypeStyle {
    let base = match role {
        TypeRole::DisplayL => TypeStyle::of(57.0, 64.0, 400, -0.25),
        TypeRole::DisplayM => TypeStyle::of(45.0, 52.0, 400, 0.0),
        TypeRole::DisplayS => TypeStyle::of(36.0, 44.0, 400, 0.0),
        TypeRole::HeadlineL => TypeStyle::of(32.0, 40.0, 400, 0.0),
        TypeRole::HeadlineM => TypeStyle::of(28.0, 36.0, 400, 0.0),
        TypeRole::HeadlineS => TypeStyle::of(24.0, 32.0, 400, 0.0),
        TypeRole::TitleL => TypeStyle::of(22.0, 28.0, 400, 0.0),
        TypeRole::TitleM => TypeStyle::of(16.0, 24.0, 500, 0.15),
        TypeRole::TitleS => TypeStyle::of(14.0, 20.0, 500, 0.1),
        TypeRole::BodyL => TypeStyle::of(16.0, 24.0, 400, 0.5),
        TypeRole::BodyM => TypeStyle::of(14.0, 20.0, 400, 0.25),
        TypeRole::BodyS => TypeStyle::of(12.0, 16.0, 400, 0.4),
        TypeRole::LabelL => TypeStyle::of(14.0, 20.0, 500, 0.1),
        TypeRole::LabelM => TypeStyle::of(12.0, 16.0, 500, 0.5),
        TypeRole::LabelS => TypeStyle::of(11.0, 16.0, 500, 0.5),
    };
    if emphasized {
        base.emphasized()
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_roles_have_positive_size() {
        let roles = [
            TypeRole::DisplayL,
            TypeRole::DisplayM,
            TypeRole::DisplayS,
            TypeRole::HeadlineL,
            TypeRole::HeadlineM,
            TypeRole::HeadlineS,
            TypeRole::TitleL,
            TypeRole::TitleM,
            TypeRole::TitleS,
            TypeRole::BodyL,
            TypeRole::BodyM,
            TypeRole::BodyS,
            TypeRole::LabelL,
            TypeRole::LabelM,
            TypeRole::LabelS,
        ];
        for r in roles {
            assert!(style(r, false).size_sp > 0.0);
            assert!(style(r, true).weight >= style(r, false).weight);
        }
    }
}
