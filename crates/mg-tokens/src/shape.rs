use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

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

/// Official catalog of 35 Material 3 Expressive shapes (`MaterialShapes` / `M3EShapeKind`).
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialShape {
    Circle = 0,
    Square = 1,
    Slanted = 2,
    Arch = 3,
    SemiCircle = 4,
    Oval = 5,
    Pill = 6,
    Triangle = 7,
    Arrow = 8,
    Fan = 9,
    Diamond = 10,
    ClamShell = 11,
    Pentagon = 12,
    Gem = 13,
    Sunny = 14,
    VerySunny = 15,
    Cookie4Sided = 16,
    Cookie6Sided = 17,
    Cookie7Sided = 18,
    Cookie9Sided = 19,
    Cookie12Sided = 20,
    Clover4Leaf = 21,
    Clover8Leaf = 22,
    Burst = 23,
    SoftBurst = 24,
    Boom = 25,
    SoftBoom = 26,
    Flower = 27,
    Puffy = 28,
    PuffyDiamond = 29,
    Ghostish = 30,
    PixelCircle = 31,
    PixelTriangle = 32,
    Bun = 33,
    Heart = 34,
}

impl MaterialShape {
    pub const ALL: [Self; 35] = [
        Self::Circle,
        Self::Square,
        Self::Slanted,
        Self::Arch,
        Self::SemiCircle,
        Self::Oval,
        Self::Pill,
        Self::Triangle,
        Self::Arrow,
        Self::Fan,
        Self::Diamond,
        Self::ClamShell,
        Self::Pentagon,
        Self::Gem,
        Self::Sunny,
        Self::VerySunny,
        Self::Cookie4Sided,
        Self::Cookie6Sided,
        Self::Cookie7Sided,
        Self::Cookie9Sided,
        Self::Cookie12Sided,
        Self::Clover4Leaf,
        Self::Clover8Leaf,
        Self::Burst,
        Self::SoftBurst,
        Self::Boom,
        Self::SoftBoom,
        Self::Flower,
        Self::Puffy,
        Self::PuffyDiamond,
        Self::Ghostish,
        Self::PixelCircle,
        Self::PixelTriangle,
        Self::Bun,
        Self::Heart,
    ];

    /// Official 7-shape morphing sequence for `LoadingIndicator` / `M3EExpressiveLoadingIndicator`.
    pub const LOADING_SEQUENCE: [Self; 7] = [
        Self::SoftBurst,
        Self::Cookie9Sided,
        Self::Pentagon,
        Self::Pill,
        Self::Sunny,
        Self::Cookie4Sided,
        Self::Oval,
    ];

    pub const fn index(self) -> u32 {
        self as u32
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Circle => "Circle",
            Self::Square => "Square",
            Self::Slanted => "Slanted",
            Self::Arch => "Arch",
            Self::SemiCircle => "SemiCircle",
            Self::Oval => "Oval",
            Self::Pill => "Pill",
            Self::Triangle => "Triangle",
            Self::Arrow => "Arrow",
            Self::Fan => "Fan",
            Self::Diamond => "Diamond",
            Self::ClamShell => "ClamShell",
            Self::Pentagon => "Pentagon",
            Self::Gem => "Gem",
            Self::Sunny => "Sunny",
            Self::VerySunny => "VerySunny",
            Self::Cookie4Sided => "Cookie4Sided",
            Self::Cookie6Sided => "Cookie6Sided",
            Self::Cookie7Sided => "Cookie7Sided",
            Self::Cookie9Sided => "Cookie9Sided",
            Self::Cookie12Sided => "Cookie12Sided",
            Self::Clover4Leaf => "Clover4Leaf",
            Self::Clover8Leaf => "Clover8Leaf",
            Self::Burst => "Burst",
            Self::SoftBurst => "SoftBurst",
            Self::Boom => "Boom",
            Self::SoftBoom => "SoftBoom",
            Self::Flower => "Flower",
            Self::Puffy => "Puffy",
            Self::PuffyDiamond => "PuffyDiamond",
            Self::Ghostish => "Ghostish",
            Self::PixelCircle => "PixelCircle",
            Self::PixelTriangle => "PixelTriangle",
            Self::Bun => "Bun",
            Self::Heart => "Heart",
        }
    }

    pub fn polygon(self) -> RoundedPolygon {
        match self {
            Self::Circle => RoundedPolygon::circle(10, 0.5, 0.5, 0.5).normalized(),
            Self::Square => {
                RoundedPolygon::rectangle(1.0, 1.0, CornerRounding::new(0.3, 0.0), None, 0.5, 0.5)
                    .normalized()
            }
            Self::Slanted => custom_polygon(
                &[
                    PointNRound::new(0.926, 0.97, 0.189, 0.811),
                    PointNRound::new(-0.021, 0.967, 0.187, 0.057),
                ],
                2,
                false,
            )
            .normalized(),
            Self::Arch => RoundedPolygon::regular(
                4,
                1.0,
                0.0,
                0.0,
                CornerRounding::UNROUNDED,
                Some(&[
                    CornerRounding::new(1.0, 0.0),
                    CornerRounding::new(1.0, 0.0),
                    CornerRounding::new(0.2, 0.0),
                    CornerRounding::new(0.2, 0.0),
                ]),
            )
            .rotated(-135.0_f32.to_radians())
            .normalized(),
            Self::SemiCircle => RoundedPolygon::rectangle(
                1.6,
                1.0,
                CornerRounding::UNROUNDED,
                Some(&[
                    CornerRounding::new(0.2, 0.0),
                    CornerRounding::new(0.2, 0.0),
                    CornerRounding::new(1.0, 0.0),
                    CornerRounding::new(1.0, 0.0),
                ]),
                0.0,
                0.0,
            )
            .normalized(),
            Self::Oval => RoundedPolygon::circle(8, 1.0, 0.0, 0.0)
                .scaled(1.0, 0.64)
                .rotated(-45.0_f32.to_radians())
                .normalized(),
            Self::Pill => custom_polygon(
                &[
                    PointNRound::new(0.961, 0.039, 0.426, 0.0),
                    PointNRound::new(1.001, 0.428, 0.0, 0.0),
                    PointNRound::new(1.0, 0.609, 1.0, 0.0),
                ],
                2,
                true,
            )
            .normalized(),
            Self::Triangle => {
                RoundedPolygon::regular(3, 1.0, 0.0, 0.0, CornerRounding::new(0.2, 0.0), None)
                    .rotated(-FRAC_PI_2)
                    .normalized()
            }
            Self::Arrow => custom_polygon(
                &[
                    PointNRound::new(0.5, 0.892, 0.313, 0.0),
                    PointNRound::new(-0.216, 1.05, 0.207, 0.0),
                    PointNRound::new(0.499, -0.16, 0.215, 1.0),
                    PointNRound::new(1.225, 1.06, 0.211, 0.0),
                ],
                1,
                false,
            )
            .normalized(),
            Self::Fan => custom_polygon(
                &[
                    PointNRound::new(1.004, 1.0, 0.148, 0.417),
                    PointNRound::new(0.0, 1.0, 0.151, 0.0),
                    PointNRound::new(0.0, -0.003, 0.148, 0.0),
                    PointNRound::new(0.978, 0.02, 0.803, 0.0),
                ],
                1,
                false,
            )
            .normalized(),
            Self::Diamond => custom_polygon(
                &[
                    PointNRound::new(0.5, 1.096, 0.151, 0.524),
                    PointNRound::new(0.04, 0.5, 0.159, 0.0),
                ],
                2,
                false,
            )
            .normalized(),
            Self::ClamShell => custom_polygon(
                &[
                    PointNRound::new(0.171, 0.841, 0.159, 0.0),
                    PointNRound::new(-0.02, 0.5, 0.14, 0.0),
                    PointNRound::new(0.17, 0.159, 0.159, 0.0),
                ],
                2,
                false,
            )
            .normalized(),
            Self::Pentagon => custom_polygon(
                &[
                    PointNRound::new(0.5, -0.009, 0.172, 0.0),
                    PointNRound::new(1.03, 0.365, 0.164, 0.0),
                    PointNRound::new(0.828, 0.97, 0.169, 0.0),
                ],
                1,
                true,
            )
            .normalized(),
            Self::Gem => custom_polygon(
                &[
                    PointNRound::new(0.499, 1.023, 0.241, 0.778),
                    PointNRound::new(-0.005, 0.792, 0.208, 0.0),
                    PointNRound::new(0.073, 0.258, 0.228, 0.0),
                    PointNRound::new(0.433, 0.0, 0.491, 0.0),
                ],
                1,
                true,
            )
            .normalized(),
            Self::Sunny => {
                RoundedPolygon::pill_star(8, 1.0, 0.8, CornerRounding::new(0.15, 0.0)).normalized()
            }
            Self::VerySunny => custom_polygon(
                &[
                    PointNRound::new(0.5, 1.08, 0.085, 0.0),
                    PointNRound::new(0.358, 0.843, 0.085, 0.0),
                ],
                8,
                false,
            )
            .normalized(),
            Self::Cookie4Sided => custom_polygon(
                &[
                    PointNRound::new(1.237, 1.236, 0.258, 0.0),
                    PointNRound::new(0.5, 0.918, 0.233, 0.0),
                ],
                4,
                false,
            )
            .normalized(),
            Self::Cookie6Sided => custom_polygon(
                &[
                    PointNRound::new(0.723, 0.884, 0.394, 0.0),
                    PointNRound::new(0.5, 1.099, 0.398, 0.0),
                ],
                6,
                false,
            )
            .normalized(),
            Self::Cookie7Sided => {
                RoundedPolygon::pill_star(7, 1.0, 0.75, CornerRounding::new(0.5, 0.0))
                    .rotated(-FRAC_PI_2)
                    .normalized()
            }
            Self::Cookie9Sided => {
                RoundedPolygon::pill_star(9, 1.0, 0.8, CornerRounding::new(0.5, 0.0))
                    .rotated(-FRAC_PI_2)
                    .normalized()
            }
            Self::Cookie12Sided => {
                RoundedPolygon::pill_star(12, 1.0, 0.8, CornerRounding::new(0.5, 0.0))
                    .rotated(-FRAC_PI_2)
                    .normalized()
            }
            Self::Clover4Leaf => custom_polygon(
                &[
                    PointNRound::new(0.5, 0.074, 0.0, 0.0),
                    PointNRound::new(0.725, -0.099, 0.476, 0.0),
                ],
                4,
                true,
            )
            .normalized(),
            Self::Clover8Leaf => custom_polygon(
                &[
                    PointNRound::new(0.5, 0.036, 0.0, 0.0),
                    PointNRound::new(0.758, -0.101, 0.209, 0.0),
                ],
                8,
                false,
            )
            .normalized(),
            Self::Burst => custom_polygon(
                &[
                    PointNRound::new(0.5, -0.006, 0.006, 0.0),
                    PointNRound::new(0.592, 0.158, 0.006, 0.0),
                ],
                12,
                false,
            )
            .normalized(),
            Self::SoftBurst => custom_polygon(
                &[
                    PointNRound::new(0.193, 0.277, 0.053, 0.0),
                    PointNRound::new(0.176, 0.055, 0.053, 0.0),
                ],
                10,
                false,
            )
            .normalized(),
            Self::Boom => custom_polygon(
                &[
                    PointNRound::new(0.457, 0.296, 0.007, 0.0),
                    PointNRound::new(0.5, -0.051, 0.007, 0.0),
                ],
                15,
                false,
            )
            .normalized(),
            Self::SoftBoom => custom_polygon(
                &[
                    PointNRound::new(0.733, 0.454, 0.0, 0.0),
                    PointNRound::new(0.839, 0.437, 0.532, 0.0),
                    PointNRound::new(0.949, 0.449, 0.439, 1.0),
                    PointNRound::new(0.998, 0.478, 0.174, 0.0),
                ],
                16,
                true,
            )
            .normalized(),
            Self::Flower => custom_polygon(
                &[
                    PointNRound::new(0.37, 0.187, 0.0, 0.0),
                    PointNRound::new(0.416, 0.049, 0.381, 0.0),
                    PointNRound::new(0.479, 0.001, 0.095, 0.0),
                ],
                8,
                true,
            )
            .normalized(),
            Self::Puffy => custom_polygon(
                &[
                    PointNRound::new(0.5, 0.053, 0.0, 0.0),
                    PointNRound::new(0.545, -0.04, 0.405, 0.0),
                    PointNRound::new(0.67, -0.035, 0.426, 0.0),
                    PointNRound::new(0.717, 0.066, 0.574, 0.0),
                    PointNRound::new(0.722, 0.128, 0.0, 0.0),
                    PointNRound::new(0.777, 0.002, 0.36, 0.0),
                    PointNRound::new(0.914, 0.149, 0.66, 0.0),
                    PointNRound::new(0.926, 0.289, 0.66, 0.0),
                    PointNRound::new(0.881, 0.346, 0.0, 0.0),
                    PointNRound::new(0.94, 0.344, 0.126, 0.0),
                    PointNRound::new(1.003, 0.437, 0.255, 0.0),
                ],
                2,
                true,
            )
            .scaled(1.0, 0.742)
            .normalized(),
            Self::PuffyDiamond => custom_polygon(
                &[
                    PointNRound::new(0.87, 0.13, 0.146, 0.0),
                    PointNRound::new(0.818, 0.357, 0.0, 0.0),
                    PointNRound::new(1.0, 0.332, 0.853, 0.0),
                ],
                4,
                true,
            )
            .normalized(),
            Self::Ghostish => custom_polygon(
                &[
                    PointNRound::new(0.5, 0.0, 1.0, 0.0),
                    PointNRound::new(1.0, 0.0, 1.0, 0.0),
                    PointNRound::new(1.0, 1.14, 0.254, 0.106),
                    PointNRound::new(0.575, 0.906, 0.253, 0.0),
                ],
                1,
                true,
            )
            .normalized(),
            Self::PixelCircle => custom_polygon(
                &[
                    PointNRound::new(0.5, 0.0, 0.0, 0.0),
                    PointNRound::new(0.704, 0.0, 0.0, 0.0),
                    PointNRound::new(0.704, 0.065, 0.0, 0.0),
                    PointNRound::new(0.843, 0.065, 0.0, 0.0),
                    PointNRound::new(0.843, 0.148, 0.0, 0.0),
                    PointNRound::new(0.926, 0.148, 0.0, 0.0),
                    PointNRound::new(0.926, 0.296, 0.0, 0.0),
                    PointNRound::new(1.0, 0.296, 0.0, 0.0),
                ],
                2,
                true,
            )
            .normalized(),
            Self::PixelTriangle => custom_polygon(
                &[
                    PointNRound::new(0.11, 0.5, 0.0, 0.0),
                    PointNRound::new(0.113, 0.0, 0.0, 0.0),
                    PointNRound::new(0.287, 0.0, 0.0, 0.0),
                    PointNRound::new(0.287, 0.087, 0.0, 0.0),
                    PointNRound::new(0.421, 0.087, 0.0, 0.0),
                    PointNRound::new(0.421, 0.17, 0.0, 0.0),
                    PointNRound::new(0.56, 0.17, 0.0, 0.0),
                    PointNRound::new(0.56, 0.265, 0.0, 0.0),
                    PointNRound::new(0.674, 0.265, 0.0, 0.0),
                    PointNRound::new(0.675, 0.344, 0.0, 0.0),
                    PointNRound::new(0.789, 0.344, 0.0, 0.0),
                    PointNRound::new(0.789, 0.439, 0.0, 0.0),
                    PointNRound::new(0.888, 0.439, 0.0, 0.0),
                ],
                1,
                true,
            )
            .normalized(),
            Self::Bun => custom_polygon(
                &[
                    PointNRound::new(0.796, 0.5, 0.0, 0.0),
                    PointNRound::new(0.853, 0.518, 1.0, 0.0),
                    PointNRound::new(0.992, 0.631, 1.0, 0.0),
                    PointNRound::new(0.968, 1.0, 1.0, 0.0),
                ],
                2,
                true,
            )
            .normalized(),
            Self::Heart => custom_polygon(
                &[
                    PointNRound::new(0.5, 0.268, 0.016, 0.0),
                    PointNRound::new(0.792, -0.066, 0.958, 0.0),
                    PointNRound::new(1.064, 0.276, 1.0, 0.0),
                    PointNRound::new(0.501, 0.946, 0.129, 0.0),
                ],
                1,
                true,
            )
            .normalized(),
        }
    }

    /// Returns 64 arc-length-parameterized contour points in `[-0.5, 0.5]` centered at origin.
    pub fn contour_64(self) -> [[f32; 2]; 64] {
        self.polygon().sample_contour_64()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn dist(self) -> f32 {
        self.x.hypot(self.y)
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn dir(self) -> Self {
        let d = self.dist();
        if d > 1e-6 {
            Self::new(self.x / d, self.y / d)
        } else {
            Self::ZERO
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    pub fn scale(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CornerRounding {
    pub radius: f32,
    pub smoothing: f32,
}

impl CornerRounding {
    pub const UNROUNDED: Self = Self {
        radius: 0.0,
        smoothing: 0.0,
    };

    pub const fn new(radius: f32, smoothing: f32) -> Self {
        Self { radius, smoothing }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointNRound {
    pub p: Point,
    pub r: CornerRounding,
}

impl PointNRound {
    pub const fn new(x: f32, y: f32, radius: f32, smoothing: f32) -> Self {
        Self {
            p: Point::new(x, y),
            r: CornerRounding::new(radius, smoothing),
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cubic {
    pub pts: [f32; 8],
}

impl Cubic {
    pub const fn new(pts: [f32; 8]) -> Self {
        Self { pts }
    }

    pub fn straight_line(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self::new([
            x0,
            y0,
            lerp(x0, x1, 1.0 / 3.0),
            lerp(y0, y1, 1.0 / 3.0),
            lerp(x0, x1, 2.0 / 3.0),
            lerp(y0, y1, 2.0 / 3.0),
            x1,
            y1,
        ])
    }

    pub fn circular_arc(cx: f32, cy: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        let p0x = x0 - cx;
        let p0y = y0 - cy;
        let p1x = x1 - cx;
        let p1y = y1 - cy;
        let d0 = Point::new(p0x, p0y).dir();
        let d1 = Point::new(p1x, p1y).dir();
        let n0x = -d0.y;
        let n0y = d0.x;
        let n1x = -d1.y;
        let n1y = d1.x;
        let cos_half = d0.dot(d1);
        if cos_half > 0.999 {
            return Self::straight_line(x0, y0, x1, y1);
        }
        let radius = p0x.hypot(p0y);
        let omc = 1.0 - cos_half;
        let s = (2.0 * omc).sqrt();
        let r = (1.0 - cos_half * cos_half).max(0.0).sqrt();
        let clockwise = if n0x * p1x + n0y * p1y >= 0.0 {
            1.0
        } else {
            -1.0
        };
        let ctl = radius * (4.0 / 3.0) * (s - r) / omc * clockwise;
        Self::new([
            x0,
            y0,
            x0 + n0x * ctl,
            y0 + n0y * ctl,
            x1 - n1x * ctl,
            y1 - n1y * ctl,
            x1,
            y1,
        ])
    }

    pub fn point_on_curve(&self, t: f32) -> Point {
        let u = 1.0 - t;
        let p = &self.pts;
        let w0 = u * u * u;
        let w1 = 3.0 * t * u * u;
        let w2 = 3.0 * t * t * u;
        let w3 = t * t * t;
        Point::new(
            p[0] * w0 + p[2] * w1 + p[4] * w2 + p[6] * w3,
            p[1] * w0 + p[3] * w1 + p[5] * w2 + p[7] * w3,
        )
    }

    pub fn bounds(&self) -> [f32; 4] {
        let p = &self.pts;
        let min_x = p[0].min(p[2]).min(p[4]).min(p[6]);
        let min_y = p[1].min(p[3]).min(p[5]).min(p[7]);
        let max_x = p[0].max(p[2]).max(p[4]).max(p[6]);
        let max_y = p[1].max(p[3]).max(p[5]).max(p[7]);
        [min_x, min_y, max_x, max_y]
    }

    pub fn transform<F: Fn(f32, f32) -> (f32, f32)>(&self, f: &F) -> Self {
        let p = &self.pts;
        let (x0, y0) = f(p[0], p[1]);
        let (x1, y1) = f(p[2], p[3]);
        let (x2, y2) = f(p[4], p[5]);
        let (x3, y3) = f(p[6], p[7]);
        Self::new([x0, y0, x1, y1, x2, y2, x3, y3])
    }
}

struct RoundedCorner {
    p0: Point,
    p1: Point,
    p2: Point,
    d1: Point,
    d2: Point,
    corner_radius: f32,
    smoothing: f32,
    expected_round_cut: f32,
}

impl RoundedCorner {
    fn new(p0: Point, p1: Point, p2: Point, rounding: CornerRounding) -> Self {
        let v01 = p0.sub(p1);
        let v21 = p2.sub(p1);
        let d01 = v01.dist();
        let d21 = v21.dist();
        if d01 > 0.0 && d21 > 0.0 {
            let d1 = v01.scale(1.0 / d01);
            let d2 = v21.scale(1.0 / d21);
            let cos_angle = d1.dot(d2);
            let sin_angle = (1.0 - cos_angle * cos_angle).max(0.0).sqrt();
            let expected_round_cut = if sin_angle > 1e-3 {
                rounding.radius * (cos_angle + 1.0) / sin_angle
            } else {
                0.0
            };
            Self {
                p0,
                p1,
                p2,
                d1,
                d2,
                corner_radius: rounding.radius,
                smoothing: rounding.smoothing,
                expected_round_cut,
            }
        } else {
            Self {
                p0,
                p1,
                p2,
                d1: Point::ZERO,
                d2: Point::ZERO,
                corner_radius: 0.0,
                smoothing: 0.0,
                expected_round_cut: 0.0,
            }
        }
    }

    fn expected_cut(&self) -> f32 {
        (1.0 + self.smoothing) * self.expected_round_cut
    }

    fn get_cubics(&self, allowed_cut0: f32, allowed_cut1: f32) -> Vec<Cubic> {
        let allowed_cut = allowed_cut0.min(allowed_cut1);
        if self.expected_round_cut < 1e-5 || allowed_cut < 1e-5 || self.corner_radius < 1e-5 {
            return vec![Cubic::straight_line(
                self.p1.x, self.p1.y, self.p1.x, self.p1.y,
            )];
        }
        let actual_round_cut = allowed_cut.min(self.expected_round_cut);
        let actual_smoothing0 = self.calc_smoothing(allowed_cut0);
        let actual_smoothing1 = self.calc_smoothing(allowed_cut1);
        let actual_r = self.corner_radius * actual_round_cut / self.expected_round_cut;
        let center_dist = actual_r.hypot(actual_round_cut);
        let bisector = self.d1.add(self.d2).scale(0.5).dir();
        let center = self.p1.add(bisector.scale(center_dist));
        let circle_inter0 = self.p1.add(self.d1.scale(actual_round_cut));
        let circle_inter2 = self.p1.add(self.d2.scale(actual_round_cut));
        let flanking0 = self.compute_flanking_curve(
            actual_round_cut,
            actual_smoothing0,
            self.p1,
            self.p0,
            circle_inter0,
            circle_inter2,
            center,
            actual_r,
        );
        let flanking2_rev = self
            .compute_flanking_curve(
                actual_round_cut,
                actual_smoothing1,
                self.p1,
                self.p2,
                circle_inter2,
                circle_inter0,
                center,
                actual_r,
            )
            .pts;
        let flanking2 = Cubic::new([
            flanking2_rev[6],
            flanking2_rev[7],
            flanking2_rev[4],
            flanking2_rev[5],
            flanking2_rev[2],
            flanking2_rev[3],
            flanking2_rev[0],
            flanking2_rev[1],
        ]);
        let arc = Cubic::circular_arc(
            center.x,
            center.y,
            flanking0.pts[6],
            flanking0.pts[7],
            flanking2.pts[0],
            flanking2.pts[1],
        );
        vec![flanking0, arc, flanking2]
    }

    fn calc_smoothing(&self, allowed_cut: f32) -> f32 {
        let exp_cut = self.expected_cut();
        if allowed_cut > exp_cut {
            self.smoothing
        } else if allowed_cut > self.expected_round_cut {
            self.smoothing * (allowed_cut - self.expected_round_cut)
                / (exp_cut - self.expected_round_cut)
        } else {
            0.0
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_flanking_curve(
        &self,
        actual_round_cut: f32,
        actual_smoothing_val: f32,
        corner: Point,
        side_start: Point,
        circle_seg_inter: Point,
        other_circle_seg_inter: Point,
        circle_center: Point,
        actual_r: f32,
    ) -> Cubic {
        let side_dir = side_start.sub(corner).dir();
        let curve_start =
            corner.add(side_dir.scale(actual_round_cut * (1.0 + actual_smoothing_val)));
        let p_mid = circle_seg_inter.add(other_circle_seg_inter).scale(0.5);
        let p = Point::new(
            lerp(circle_seg_inter.x, p_mid.x, actual_smoothing_val),
            lerp(circle_seg_inter.y, p_mid.y, actual_smoothing_val),
        );
        let curve_end = circle_center.add(p.sub(circle_center).dir().scale(actual_r));
        let circle_tan = Point::new(
            -(curve_end.y - circle_center.y),
            curve_end.x - circle_center.x,
        );
        let anchor_end = Self::line_intersection(side_start, side_dir, curve_end, circle_tan)
            .unwrap_or(circle_seg_inter);
        let anchor_start = curve_start.add(anchor_end.scale(2.0)).scale(1.0 / 3.0);
        Cubic::new([
            curve_start.x,
            curve_start.y,
            anchor_start.x,
            anchor_start.y,
            anchor_end.x,
            anchor_end.y,
            curve_end.x,
            curve_end.y,
        ])
    }

    fn line_intersection(p0: Point, d0: Point, p1: Point, d1: Point) -> Option<Point> {
        let n1 = Point::new(-d1.y, d1.x);
        let den = d0.dot(n1);
        if den.abs() < 1e-5 {
            return None;
        }
        let num = p1.sub(p0).dot(n1);
        if den.abs() < 1e-5 * num.abs() {
            return None;
        }
        Some(p0.add(d0.scale(num / den)))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoundedPolygon {
    pub cubics: Vec<Cubic>,
    pub center: Point,
}

impl RoundedPolygon {
    pub fn from_vertices(
        vertices: &[f32],
        center_x: f32,
        center_y: f32,
        per_vertex_rounding: Option<&[CornerRounding]>,
        rounding: CornerRounding,
    ) -> Self {
        let n = vertices.len() / 2;
        let mut corners = Vec::with_capacity(n);
        for i in 0..n {
            let r = per_vertex_rounding
                .and_then(|list| list.get(i).copied())
                .unwrap_or(rounding);
            let prev_i = (i + n - 1) % n;
            let next_i = (i + 1) % n;
            corners.push(RoundedCorner::new(
                Point::new(vertices[prev_i * 2], vertices[prev_i * 2 + 1]),
                Point::new(vertices[i * 2], vertices[i * 2 + 1]),
                Point::new(vertices[next_i * 2], vertices[next_i * 2 + 1]),
                r,
            ));
        }
        let mut cut_adjusts = Vec::with_capacity(n);
        for i in 0..n {
            let rc0 = &corners[i];
            let rc1 = &corners[(i + 1) % n];
            let r_req = rc0.expected_round_cut + rc1.expected_round_cut;
            let s_req = rc0.expected_cut() + rc1.expected_cut();
            let next_i = (i + 1) % n;
            let dx = vertices[i * 2] - vertices[next_i * 2];
            let dy = vertices[i * 2 + 1] - vertices[next_i * 2 + 1];
            let side_len = dx.hypot(dy);
            if r_req > side_len && r_req > 0.0 {
                cut_adjusts.push((side_len / r_req, 0.0));
            } else if s_req > side_len && (s_req - r_req) > 0.0 {
                cut_adjusts.push((1.0, (side_len - r_req) / (s_req - r_req)));
            } else {
                cut_adjusts.push((1.0, 1.0));
            }
        }
        let mut corner_cubics = Vec::with_capacity(n);
        for i in 0..n {
            let mut allowed = [0.0f32; 2];
            for (delta, slot) in allowed.iter_mut().enumerate() {
                let (rr, ss) = cut_adjusts[(i + n - 1 + delta) % n];
                let rc = &corners[i];
                *slot =
                    rc.expected_round_cut * rr + (rc.expected_cut() - rc.expected_round_cut) * ss;
            }
            corner_cubics.push(corners[i].get_cubics(allowed[0], allowed[1]));
        }
        let mut raw_cubics = Vec::with_capacity(n * 4);
        for i in 0..n {
            for c in &corner_cubics[i] {
                raw_cubics.push(*c);
            }
            let last_c = corner_cubics[i].last().expect("corner cubics non-empty");
            let next_c = corner_cubics[(i + 1) % n]
                .first()
                .expect("corner cubics non-empty");
            raw_cubics.push(Cubic::straight_line(
                last_c.pts[6],
                last_c.pts[7],
                next_c.pts[0],
                next_c.pts[1],
            ));
        }
        let cubics: Vec<Cubic> = raw_cubics
            .into_iter()
            .filter(|c| (c.pts[0] - c.pts[6]).hypot(c.pts[1] - c.pts[7]) >= 1e-5)
            .collect();
        Self {
            cubics,
            center: Point::new(center_x, center_y),
        }
    }

    pub fn circle(num_vertices: usize, radius: f32, center_x: f32, center_y: f32) -> Self {
        let r = radius / (PI / num_vertices as f32).cos();
        Self::regular(
            num_vertices,
            r,
            center_x,
            center_y,
            CornerRounding::new(radius, 0.0),
            None,
        )
    }

    pub fn regular(
        num_vertices: usize,
        radius: f32,
        center_x: f32,
        center_y: f32,
        rounding: CornerRounding,
        per_vertex_rounding: Option<&[CornerRounding]>,
    ) -> Self {
        let mut verts = Vec::with_capacity(num_vertices * 2);
        for i in 0..num_vertices {
            let a = TAU * (i as f32) / (num_vertices as f32);
            verts.push(a.cos() * radius + center_x);
            verts.push(a.sin() * radius + center_y);
        }
        Self::from_vertices(&verts, center_x, center_y, per_vertex_rounding, rounding)
    }

    pub fn pill_star(
        num_vertices_per_radius: usize,
        radius: f32,
        inner_radius: f32,
        rounding: CornerRounding,
    ) -> Self {
        let mut verts = Vec::with_capacity(num_vertices_per_radius * 4);
        let step = PI / (num_vertices_per_radius as f32);
        for i in 0..num_vertices_per_radius {
            let a0 = 2.0 * step * (i as f32);
            verts.push(a0.cos() * radius);
            verts.push(a0.sin() * radius);
            let a1 = step * (2 * i + 1) as f32;
            verts.push(a1.cos() * inner_radius);
            verts.push(a1.sin() * inner_radius);
        }
        Self::from_vertices(&verts, 0.0, 0.0, None, rounding)
    }

    pub fn rectangle(
        width: f32,
        height: f32,
        rounding: CornerRounding,
        per_vertex_rounding: Option<&[CornerRounding]>,
        center_x: f32,
        center_y: f32,
    ) -> Self {
        let hw = width * 0.5;
        let hh = height * 0.5;
        let l = center_x - hw;
        let t = center_y - hh;
        let r = center_x + hw;
        let b = center_y + hh;
        Self::from_vertices(
            &[r, b, l, b, l, t, r, t],
            center_x,
            center_y,
            per_vertex_rounding,
            rounding,
        )
    }

    pub fn transformed<F: Fn(f32, f32) -> (f32, f32)>(&self, f: F) -> Self {
        let (cx, cy) = f(self.center.x, self.center.y);
        let cubics = self.cubics.iter().map(|c| c.transform(&f)).collect();
        Self {
            cubics,
            center: Point::new(cx, cy),
        }
    }

    pub fn rotated(&self, radians: f32) -> Self {
        let (sn, cs) = radians.sin_cos();
        self.transformed(|x, y| (x * cs - y * sn, x * sn + y * cs))
    }

    pub fn scaled(&self, sx: f32, sy: f32) -> Self {
        self.transformed(|x, y| (x * sx, y * sy))
    }

    pub fn bounds(&self) -> [f32; 4] {
        let mut b = [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ];
        for c in &self.cubics {
            let cb = c.bounds();
            b[0] = b[0].min(cb[0]);
            b[1] = b[1].min(cb[1]);
            b[2] = b[2].max(cb[2]);
            b[3] = b[3].max(cb[3]);
        }
        b
    }

    pub fn normalized(&self) -> Self {
        let b = self.bounds();
        let w = (b[2] - b[0]).max(1e-5);
        let h = (b[3] - b[1]).max(1e-5);
        let side = w.max(h);
        let ox = (side - w) * 0.5 - b[0];
        let oy = (side - h) * 0.5 - b[1];
        self.transformed(|x, y| ((x + ox) / side, (y + oy) / side))
    }

    /// Samples 64 clockwise arc-length-parameterized points in `[-0.5, 0.5]` starting from `-PI/2`,
    /// snapping to sharp corners when present (e.g., `PixelCircle` and `PixelTriangle`).
    pub fn sample_contour_64(&self) -> [[f32; 2]; 64] {
        let mut dense = Vec::with_capacity(self.cubics.len() * 12);
        for c in &self.cubics {
            let p = &c.pts;
            let dx = p[6] - p[0];
            let dy = p[7] - p[1];
            let is_line = ((p[2] - p[0]) * dy - (p[3] - p[1]) * dx).abs() < 1e-4
                && ((p[4] - p[0]) * dy - (p[5] - p[1]) * dx).abs() < 1e-4;
            let steps = if is_line { 2 } else { 12 };
            for s in 0..steps {
                let pt = c.point_on_curve(s as f32 / steps as f32);
                dense.push([pt.x - 0.5, pt.y - 0.5]);
            }
        }
        if dense.is_empty() {
            return [[0.0, 0.0]; 64];
        }
        let mut signed_area = 0.0f32;
        for i in 0..dense.len() {
            let [x0, y0] = dense[i];
            let [x1, y1] = dense[(i + 1) % dense.len()];
            signed_area += x0 * y1 - x1 * y0;
        }
        if signed_area < 0.0 {
            dense.reverse();
        }
        let mut best_i = 0usize;
        let mut best_ang_diff = f32::INFINITY;
        for (i, &[x, y]) in dense.iter().enumerate() {
            let ang = y.atan2(x);
            let diff = (ang - (-FRAC_PI_2)).abs();
            if diff < best_ang_diff {
                best_ang_diff = diff;
                best_i = i;
            }
        }
        dense.rotate_left(best_i);

        let n_d = dense.len();
        let mut sharp = vec![false; n_d];
        for i in 0..n_d {
            let p_prev = dense[(i + n_d - 1) % n_d];
            let p_curr = dense[i];
            let p_next = dense[(i + 1) % n_d];
            let v1x = p_curr[0] - p_prev[0];
            let v1y = p_curr[1] - p_prev[1];
            let v2x = p_next[0] - p_curr[0];
            let v2y = p_next[1] - p_curr[1];
            let l1 = v1x.hypot(v1y);
            let l2 = v2x.hypot(v2y);
            if l1 > 1e-5 && l2 > 1e-5 {
                let cos_a = (v1x * v2x + v1y * v2y) / (l1 * l2);
                if cos_a < 0.85 {
                    sharp[i] = true;
                }
            }
        }

        let mut cum = Vec::with_capacity(n_d + 1);
        cum.push(0.0f32);
        for i in 0..n_d {
            let [x0, y0] = dense[i];
            let [x1, y1] = dense[(i + 1) % n_d];
            let prev = *cum.last().unwrap_or(&0.0);
            cum.push(prev + (x1 - x0).hypot(y1 - y0));
        }
        let total = (*cum.last().unwrap_or(&1.0)).max(1e-6);
        let step_arc = total / 64.0;
        let mut out = [[0.0f32; 2]; 64];
        let mut idx = 0usize;
        for (k, slot) in out.iter_mut().enumerate() {
            let target = (k as f32) * step_arc;
            while idx + 1 < cum.len() && cum[idx + 1] < target {
                idx += 1;
            }
            let seg_len = if idx + 1 < cum.len() {
                cum[idx + 1] - cum[idx]
            } else {
                0.0
            };
            let t = if seg_len > 1e-6 {
                (target - cum[idx]) / seg_len
            } else {
                0.0
            };
            let i0 = idx % n_d;
            let i1 = (idx + 1) % n_d;
            if sharp[i0] && (target - cum[idx]) < step_arc * 0.55 {
                *slot = dense[i0];
            } else if idx + 1 < cum.len() && sharp[i1] && (cum[idx + 1] - target) <= step_arc * 0.55
            {
                *slot = dense[i1];
            } else {
                let [x0, y0] = dense[i0];
                let [x1, y1] = dense[i1];
                *slot = [lerp(x0, x1, t), lerp(y0, y1, t)];
            }
        }
        out
    }
}

pub fn custom_polygon(pnr: &[PointNRound], reps: usize, mirroring: bool) -> RoundedPolygon {
    let cx = 0.5f32;
    let cy = 0.5f32;
    let n = pnr.len();
    let mut pts = Vec::new();
    if mirroring {
        let mut angles = Vec::with_capacity(n);
        let mut dists = Vec::with_capacity(n);
        for item in pnr {
            let dx = item.p.x - cx;
            let dy = item.p.y - cy;
            angles.push(dy.atan2(dx));
            dists.push(dx.hypot(dy));
        }
        let r = reps * 2;
        let section = TAU / (r as f32);
        for it in 0..r {
            let even = (it % 2) == 0;
            let base_ang = section * (it as f32);
            for k in 0..n {
                let i = if even { k } else { n - 1 - k };
                if i > 0 || even {
                    let ang = if even {
                        angles[i]
                    } else {
                        section - angles[i] + 2.0 * angles[0]
                    };
                    let f = base_ang + ang;
                    let d = dists[i];
                    pts.push(PointNRound::new(
                        f.cos() * d + cx,
                        f.sin() * d + cy,
                        pnr[i].r.radius,
                        pnr[i].r.smoothing,
                    ));
                }
            }
        }
    } else {
        let total = n * reps;
        for i in 0..total {
            let idx = i % n;
            let rep = i / n;
            let ang = (rep as f32) * TAU / (reps as f32);
            let dx = pnr[idx].p.x - cx;
            let dy = pnr[idx].p.y - cy;
            let (sn, cs) = ang.sin_cos();
            pts.push(PointNRound::new(
                dx * cs - dy * sn + cx,
                dx * sn + dy * cs + cy,
                pnr[idx].r.radius,
                pnr[idx].r.smoothing,
            ));
        }
    }
    let mut verts = Vec::with_capacity(pts.len() * 2);
    let mut roundings = Vec::with_capacity(pts.len());
    for p in pts {
        verts.push(p.p.x);
        verts.push(p.p.y);
        roundings.push(p.r);
    }
    RoundedPolygon::from_vertices(&verts, cx, cy, Some(&roundings), CornerRounding::UNROUNDED)
}

/// Morphs smoothly between two `MaterialShape` contours.
#[derive(Clone, Debug)]
pub struct Morph {
    pub start: [[f32; 2]; 64],
    pub end: [[f32; 2]; 64],
}

impl Morph {
    pub fn new(start: MaterialShape, end: MaterialShape) -> Self {
        Self {
            start: start.contour_64(),
            end: end.contour_64(),
        }
    }

    pub fn evaluate(&self, progress: f32) -> [[f32; 2]; 64] {
        let t = progress.clamp(0.0, 1.0);
        let mut out = [[0.0f32; 2]; 64];
        for (i, o) in out.iter_mut().enumerate() {
            *o = [
                lerp(self.start[i][0], self.end[i][0], t),
                lerp(self.start[i][1], self.end[i][1], t),
            ];
        }
        out
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

    #[test]
    fn all_35_material_shapes_build_and_normalize() {
        assert_eq!(MaterialShape::ALL.len(), 35);
        for (idx, shape) in MaterialShape::ALL.iter().copied().enumerate() {
            assert_eq!(shape.index() as usize, idx);
            let poly = shape.polygon();
            assert!(!poly.cubics.is_empty(), "{} has no cubics", shape.name());
            let b = poly.bounds();
            assert!(
                b[0] >= -0.05 && b[1] >= -0.05 && b[2] <= 1.05 && b[3] <= 1.05,
                "{} bounds out of [0,1]: {:?}",
                shape.name(),
                b
            );
            let contour = shape.contour_64();
            assert_eq!(contour.len(), 64);
        }
    }

    #[test]
    fn loading_sequence_morphs_cleanly() {
        assert_eq!(MaterialShape::LOADING_SEQUENCE.len(), 7);
        for i in 0..MaterialShape::LOADING_SEQUENCE.len() {
            let a = MaterialShape::LOADING_SEQUENCE[i];
            let b = MaterialShape::LOADING_SEQUENCE[(i + 1) % 7];
            let morph = Morph::new(a, b);
            let mid = morph.evaluate(0.5);
            assert!(mid.iter().any(|p| p[0].hypot(p[1]) > 0.2));
        }
    }
}
