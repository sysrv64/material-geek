use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Argb(pub u32);

impl Argb {
    pub const fn new(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(0xFF, r, g, b)
    }
    pub const fn hex(rgb: u32) -> Self {
        Self(0xFF00_0000 | rgb)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorScheme {
    pub is_dark: bool,
    pub primary: Argb,
    pub on_primary: Argb,
    pub primary_container: Argb,
    pub on_primary_container: Argb,
    pub inverse_primary: Argb,
    pub secondary: Argb,
    pub on_secondary: Argb,
    pub secondary_container: Argb,
    pub on_secondary_container: Argb,
    pub tertiary: Argb,
    pub on_tertiary: Argb,
    pub tertiary_container: Argb,
    pub on_tertiary_container: Argb,
    pub error: Argb,
    pub on_error: Argb,
    pub error_container: Argb,
    pub on_error_container: Argb,
    pub background: Argb,
    pub on_background: Argb,
    pub surface: Argb,
    pub on_surface: Argb,
    pub surface_variant: Argb,
    pub on_surface_variant: Argb,
    pub surface_tint: Argb,
    pub surface_dim: Argb,
    pub surface_bright: Argb,
    pub surface_container_lowest: Argb,
    pub surface_container_low: Argb,
    pub surface_container: Argb,
    pub surface_container_high: Argb,
    pub surface_container_highest: Argb,
    pub inverse_surface: Argb,
    pub inverse_on_surface: Argb,
    pub outline: Argb,
    pub outline_variant: Argb,
    pub scrim: Argb,
    pub shadow: Argb,
    pub primary_fixed: Argb,
    pub primary_fixed_dim: Argb,
    pub on_primary_fixed: Argb,
    pub on_primary_fixed_variant: Argb,
    pub secondary_fixed: Argb,
    pub secondary_fixed_dim: Argb,
    pub on_secondary_fixed: Argb,
    pub on_secondary_fixed_variant: Argb,
    pub tertiary_fixed: Argb,
    pub tertiary_fixed_dim: Argb,
    pub on_tertiary_fixed: Argb,
    pub on_tertiary_fixed_variant: Argb,
    pub error_dim: Argb,
}

impl ColorScheme {
    pub fn light() -> Self {
        Self {
            is_dark: false,
            primary: Argb::hex(0x6750A4),
            on_primary: Argb::hex(0xFFFFFF),
            primary_container: Argb::hex(0xEADDFF),
            on_primary_container: Argb::hex(0x4F378B),
            inverse_primary: Argb::hex(0xD0BCFF),
            secondary: Argb::hex(0x625B71),
            on_secondary: Argb::hex(0xFFFFFF),
            secondary_container: Argb::hex(0xE8DEF8),
            on_secondary_container: Argb::hex(0x4A4458),
            tertiary: Argb::hex(0x7D5260),
            on_tertiary: Argb::hex(0xFFFFFF),
            tertiary_container: Argb::hex(0xFFD8E4),
            on_tertiary_container: Argb::hex(0x633B48),
            error: Argb::hex(0xB3261E),
            on_error: Argb::hex(0xFFFFFF),
            error_container: Argb::hex(0xF9DEDC),
            on_error_container: Argb::hex(0x410002),
            background: Argb::hex(0xFEF7FF),
            on_background: Argb::hex(0x1D1B20),
            surface: Argb::hex(0xFEF7FF),
            on_surface: Argb::hex(0x1D1B20),
            surface_variant: Argb::hex(0xE7E0EC),
            on_surface_variant: Argb::hex(0x49454F),
            surface_tint: Argb::hex(0x6750A4),
            surface_dim: Argb::hex(0xDED8E1),
            surface_bright: Argb::hex(0xFEF7FF),
            surface_container_lowest: Argb::hex(0xFFFFFF),
            surface_container_low: Argb::hex(0xF7F2FA),
            surface_container: Argb::hex(0xF3EDF7),
            surface_container_high: Argb::hex(0xECE6F0),
            surface_container_highest: Argb::hex(0xE6E0E9),
            inverse_surface: Argb::hex(0x322F35),
            inverse_on_surface: Argb::hex(0xF5EFF7),
            outline: Argb::hex(0x79747E),
            outline_variant: Argb::hex(0xCAC4D0),
            scrim: Argb::hex(0x000000),
            shadow: Argb::hex(0x000000),
            primary_fixed: Argb::hex(0xEADDFF),
            primary_fixed_dim: Argb::hex(0xD0BCFF),
            on_primary_fixed: Argb::hex(0x4F378B),
            on_primary_fixed_variant: Argb::hex(0x4F378B),
            secondary_fixed: Argb::hex(0xE8DEF8),
            secondary_fixed_dim: Argb::hex(0xCCC2DC),
            on_secondary_fixed: Argb::hex(0x4A4458),
            on_secondary_fixed_variant: Argb::hex(0x4A4458),
            tertiary_fixed: Argb::hex(0xFFD8E4),
            tertiary_fixed_dim: Argb::hex(0xEFB8C8),
            on_tertiary_fixed: Argb::hex(0x633B48),
            on_tertiary_fixed_variant: Argb::hex(0x633B48),
            error_dim: Argb::hex(0xEFB8C8),
        }
    }

    pub fn dark() -> Self {
        Self {
            is_dark: true,
            primary: Argb::hex(0xD0BCFF),
            on_primary: Argb::hex(0x381E72),
            primary_container: Argb::hex(0x4F378B),
            on_primary_container: Argb::hex(0xEADDFF),
            inverse_primary: Argb::hex(0x6750A4),
            secondary: Argb::hex(0xCCC2DC),
            on_secondary: Argb::hex(0x332D41),
            secondary_container: Argb::hex(0x4A4458),
            on_secondary_container: Argb::hex(0xE8DEF8),
            tertiary: Argb::hex(0xEFB8C8),
            on_tertiary: Argb::hex(0x492532),
            tertiary_container: Argb::hex(0x633B48),
            on_tertiary_container: Argb::hex(0xFFD8E4),
            error: Argb::hex(0xF2B8B5),
            on_error: Argb::hex(0x601410),
            error_container: Argb::hex(0x8C1D18),
            on_error_container: Argb::hex(0xF9DEDC),
            background: Argb::hex(0x141218),
            on_background: Argb::hex(0xE6E0E9),
            surface: Argb::hex(0x141218),
            on_surface: Argb::hex(0xE6E0E9),
            surface_variant: Argb::hex(0x49454F),
            on_surface_variant: Argb::hex(0xCAC4D0),
            surface_tint: Argb::hex(0xD0BCFF),
            surface_dim: Argb::hex(0x141218),
            surface_bright: Argb::hex(0x3B383E),
            surface_container_lowest: Argb::hex(0x0F0D13),
            surface_container_low: Argb::hex(0x1D1B20),
            surface_container: Argb::hex(0x211F26),
            surface_container_high: Argb::hex(0x2B2930),
            surface_container_highest: Argb::hex(0x36343B),
            inverse_surface: Argb::hex(0xE6E0E9),
            inverse_on_surface: Argb::hex(0x322F35),
            outline: Argb::hex(0x938F99),
            outline_variant: Argb::hex(0x49454F),
            scrim: Argb::hex(0x000000),
            shadow: Argb::hex(0x000000),
            primary_fixed: Argb::hex(0xEADDFF),
            primary_fixed_dim: Argb::hex(0xD0BCFF),
            on_primary_fixed: Argb::hex(0x4F378B),
            on_primary_fixed_variant: Argb::hex(0x381E72),
            secondary_fixed: Argb::hex(0xE8DEF8),
            secondary_fixed_dim: Argb::hex(0xCCC2DC),
            on_secondary_fixed: Argb::hex(0x4A4458),
            on_secondary_fixed_variant: Argb::hex(0x332D41),
            tertiary_fixed: Argb::hex(0xFFD8E4),
            tertiary_fixed_dim: Argb::hex(0xEFB8C8),
            on_tertiary_fixed: Argb::hex(0x633B48),
            on_tertiary_fixed_variant: Argb::hex(0x492532),
            error_dim: Argb::hex(0x8C1D18),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_dark_differ() {
        assert_ne!(ColorScheme::light(), ColorScheme::dark());
        assert!(!ColorScheme::light().is_dark);
        assert!(ColorScheme::dark().is_dark);
    }

    #[test]
    fn fixed_dim_roles_present() {
        let l = ColorScheme::light();
        assert_ne!(l.primary_fixed, l.primary_fixed_dim);
    }
}
