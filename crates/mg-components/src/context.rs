use crate::props::TextFieldProps;
use mg_motion::{cubic_bezier_y, DynamicSpring, Tempo, Track};
use mg_render::{
    m3_text_style, MaterialShape, TextStyle, TypeStyle, UiRenderer, M3_LABEL_LARGE,
    M3_LABEL_MEDIUM, M3_TITLE_MEDIUM,
};
use mg_theme::GeekTheme;
use mg_tokens::color::{Argb, ColorScheme};
use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::hash::{Hash, Hasher};

pub fn srgb(byte: u8) -> f32 {
    byte as f32 / 255.0
}

pub fn argb_hex(argb: u32) -> [f32; 4] {
    [
        srgb(((argb >> 16) & 0xFF) as u8),
        srgb(((argb >> 8) & 0xFF) as u8),
        srgb((argb & 0xFF) as u8),
        1.0,
    ]
}

pub fn argb_to_f32(c: Argb) -> [f32; 4] {
    let a = ((c.0 >> 24) & 0xFF) as f32 / 255.0;
    let r = ((c.0 >> 16) & 0xFF) as f32 / 255.0;
    let g = ((c.0 >> 8) & 0xFF) as f32 / 255.0;
    let b = (c.0 & 0xFF) as f32 / 255.0;
    [r, g, b, a]
}

pub fn r4(r: f32) -> [f32; 4] {
    [r, r, r, r]
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn with_alpha(c: [f32; 4], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], c[3] * a]
}

pub fn mix(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

pub fn ts(t: TypeStyle, color: [f32; 4], u: f32) -> TextStyle {
    let mut s = m3_text_style(t, color);
    s.px *= u;
    s.line_h *= u;
    s
}

pub fn icon_c(ui: &mut UiRenderer, name: &str, cx: f32, cy: f32, px: f32, color: [f32; 4]) {
    ui.icon(name, cx - px * 0.5, cy - px * 0.5, px, color);
}

pub fn text_c(ui: &mut UiRenderer, s: &str, cx: f32, y_top: f32, style: TextStyle) {
    let tw = ui.text_width(s, style.px, style.medium, style.tracking_em);
    ui.text(s, cx - tw * 0.5, y_top, style);
}

/// Evaluates one timed segment of a linear indeterminate cycle (`M3EProgressIndicatorUtils.evaluateIndeterminateSegment`).
pub fn eval_indet_segment(t_cycle: f32, delay_ms: f32, duration_ms: f32, total_ms: f32) -> f32 {
    let start = delay_ms / total_ms;
    let end = (delay_ms + duration_ms) / total_ms;
    if t_cycle <= start {
        0.0
    } else if t_cycle >= end {
        1.0
    } else {
        let local_t = (t_cycle - start) / (end - start);
        cubic_bezier_y(local_t, 0.3, 0.0, 0.8, 0.15)
    }
}

/// Evaluates the M3 Expressive 7-shape loading sequence (`SoftBurst -> Cookie9Sided -> Pentagon -> Pill -> Sunny -> Cookie4Sided -> Oval`)
/// returning `(shape_a, shape_b, morph_t, rotation_rad, scale_factor)`.
pub fn loading_morph_state(
    t: f32,
    phase_offset: usize,
) -> (MaterialShape, MaterialShape, f32, f32, f32) {
    let seq = &MaterialShape::LOADING_SEQUENCE;
    let cycle_dur = 0.65;
    let total = t / cycle_dur + phase_offset as f32;
    let step = total.floor() as usize;
    let local = total.fract();
    let morph_t = if local < 0.68 {
        let u = local / 0.68;
        cubic_bezier_y(u, 0.22, 1.0, 0.36, 1.0)
    } else {
        1.0
    };
    let shape_a = seq[step % seq.len()];
    let shape_b = seq[(step + 1) % seq.len()];
    let rot = t * (TAU / 4.666) + (step as f32 + morph_t) * FRAC_PI_2;
    let scale = 1.0 - 0.06 * (morph_t * PI).sin();
    (shape_a, shape_b, morph_t, rot, scale)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }

    pub fn cx(self) -> f32 {
        self.x + self.w * 0.5
    }

    pub fn cy(self) -> f32 {
        self.y + self.h * 0.5
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

impl WidgetId {
    pub fn from_label(label: &str) -> Self {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        label.hash(&mut h);
        Self(h.finish())
    }

    pub fn with_index(self, idx: usize) -> Self {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.0.hash(&mut h);
        idx.hash(&mut h);
        Self(h.finish())
    }
}

impl From<&str> for WidgetId {
    fn from(s: &str) -> Self {
        Self::from_label(s)
    }
}

impl From<u32> for WidgetId {
    fn from(v: u32) -> Self {
        Self(v as u64)
    }
}

impl From<u64> for WidgetId {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WidgetResponse {
    pub clicked: bool,
    pub pressed: bool,
    pub dragged: bool,
    pub focused: bool,
    pub changed: bool,
    pub drag_delta_x: f32,
    pub drag_delta_y: f32,
    pub rect: Rect,
}

#[derive(Clone, Debug)]
pub struct Palette {
    pub is_dark: bool,
    pub bg: [f32; 4],
    pub surface_low: [f32; 4],
    pub card: [f32; 4],
    pub card_hi: [f32; 4],
    pub card_highest: [f32; 4],
    pub surface_container_low: [f32; 4],
    pub surface_container: [f32; 4],
    pub surface_container_high: [f32; 4],
    pub surface_container_highest: [f32; 4],
    pub scrim: [f32; 4],
    pub text: [f32; 4],
    pub sub: [f32; 4],
    pub primary: [f32; 4],
    pub on_primary: [f32; 4],
    pub primary_container: [f32; 4],
    pub on_primary_container: [f32; 4],
    pub secondary: [f32; 4],
    pub secondary_container: [f32; 4],
    pub on_secondary_container: [f32; 4],
    pub tertiary: [f32; 4],
    pub tertiary_container: [f32; 4],
    pub on_tertiary_container: [f32; 4],
    pub error: [f32; 4],
    pub on_error: [f32; 4],
    pub error_container: [f32; 4],
    pub on_error_container: [f32; 4],
    pub outline: [f32; 4],
    pub outline_variant: [f32; 4],
    pub inverse: [f32; 4],
    pub on_inverse: [f32; 4],
    pub inverse_primary: [f32; 4],
}

impl Palette {
    pub fn from_scheme(s: &ColorScheme) -> Self {
        Self {
            is_dark: s.is_dark,
            bg: argb_to_f32(s.background),
            surface_low: argb_to_f32(s.surface_container_low),
            card: argb_to_f32(s.surface_container),
            card_hi: argb_to_f32(s.surface_container_high),
            card_highest: argb_to_f32(s.surface_container_highest),
            surface_container_low: argb_to_f32(s.surface_container_low),
            surface_container: argb_to_f32(s.surface_container),
            surface_container_high: argb_to_f32(s.surface_container_high),
            surface_container_highest: argb_to_f32(s.surface_container_highest),
            scrim: argb_to_f32(s.scrim),
            text: argb_to_f32(s.on_surface),
            sub: argb_to_f32(s.on_surface_variant),
            primary: argb_to_f32(s.primary),
            on_primary: argb_to_f32(s.on_primary),
            primary_container: argb_to_f32(s.primary_container),
            on_primary_container: argb_to_f32(s.on_primary_container),
            secondary: argb_to_f32(s.secondary),
            secondary_container: argb_to_f32(s.secondary_container),
            on_secondary_container: argb_to_f32(s.on_secondary_container),
            tertiary: argb_to_f32(s.tertiary),
            tertiary_container: argb_to_f32(s.tertiary_container),
            on_tertiary_container: argb_to_f32(s.on_tertiary_container),
            error: argb_to_f32(s.error),
            on_error: argb_to_f32(s.on_error),
            error_container: argb_to_f32(s.error_container),
            on_error_container: argb_to_f32(s.on_error_container),
            outline: argb_to_f32(s.outline),
            outline_variant: argb_to_f32(s.outline_variant),
            inverse: argb_to_f32(s.inverse_surface),
            on_inverse: argb_to_f32(s.inverse_on_surface),
            inverse_primary: argb_to_f32(s.inverse_primary),
        }
    }

    pub fn dark() -> Self {
        Self::from_scheme(&ColorScheme::baseline_dark())
    }

    pub fn light() -> Self {
        Self::from_scheme(&ColorScheme::baseline_light())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyInput {
    Char(char),
    Text(String),
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Enter,
    Escape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoftKeyboardMode {
    Qwerty,
    Numeric,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ZoneKind {
    Tap { bounded: bool },
    ImmediateHorizontalDrag,
    SwipeHorizontalDrag,
    VerticalPullDrag,
    DialDrag { cx: f32, cy: f32 },
}

#[derive(Clone, Copy, Debug)]
pub struct HitZone {
    pub id: WidgetId,
    pub rect: Rect,
    pub ripple_color: [f32; 4],
    pub kind: ZoneKind,
    pub overlay: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Ripple {
    pub id: WidgetId,
    pub cx: f32,
    pub cy: f32,
    pub rect: Rect,
    pub color: [f32; 4],
    pub start_t: f32,
    pub release_t: Option<f32>,
    pub bounded: bool,
}

pub struct UiState {
    pub theme: GeekTheme,
    pub pal: Palette,
    pub t: f32,
    pub scroll: f32,
    pub content_h: f32,
    pub width: f32,
    pub height: f32,
    pub pointer: (f32, f32),
    pub press_start: (f32, f32),
    pub prev_pointer: (f32, f32),
    pub frame_delta: (f32, f32),
    pub is_down: bool,
    pub dragging_scroll: bool,
    pub active_zone: Option<HitZone>,
    pub active_drag_id: Option<WidgetId>,
    pub just_clicked: Option<WidgetId>,
    pub focus_id: Option<WidgetId>,
    pub soft_kb_open: bool,
    pub soft_kb_mode: SoftKeyboardMode,
    pub soft_kb_shift: bool,
    pub soft_kb_symbols: bool,
    pub key_queue: Vec<KeyInput>,
    pub zones: Vec<HitZone>,
    pub ripples: Vec<Ripple>,
    pub springs: HashMap<(WidgetId, u8), DynamicSpring>,
}

impl Default for UiState {
    fn default() -> Self {
        Self::new(GeekTheme::dark_expressive())
    }
}

impl UiState {
    pub fn new(theme: GeekTheme) -> Self {
        let pal = Palette::from_scheme(&theme.scheme);
        Self {
            theme,
            pal,
            t: 0.0,
            scroll: 0.0,
            content_h: 4000.0,
            width: 412.0,
            height: 892.0,
            pointer: (0.0, 0.0),
            press_start: (0.0, 0.0),
            prev_pointer: (0.0, 0.0),
            frame_delta: (0.0, 0.0),
            is_down: false,
            dragging_scroll: false,
            active_zone: None,
            active_drag_id: None,
            just_clicked: None,
            focus_id: None,
            soft_kb_open: false,
            soft_kb_mode: SoftKeyboardMode::Qwerty,
            soft_kb_shift: false,
            soft_kb_symbols: false,
            key_queue: Vec::new(),
            zones: Vec::with_capacity(256),
            ripples: Vec::with_capacity(16),
            springs: HashMap::with_capacity(128),
        }
    }

    pub fn u(&self) -> f32 {
        (self.width / 412.0).clamp(0.75, 3.5)
    }

    pub fn toggle_dark_mode(&mut self) {
        let next_dark = !self.theme.scheme.is_dark;
        let scheme = if next_dark {
            ColorScheme::baseline_dark()
        } else {
            ColorScheme::baseline_light()
        };
        self.pal = Palette::from_scheme(&scheme);
        self.theme.scheme = scheme;
    }

    pub fn toggle_motion_scheme(&mut self) {
        self.theme.expressive_motion = !self.theme.expressive_motion;
    }

    pub fn focus(&mut self, id: impl Into<WidgetId>, mode: SoftKeyboardMode) {
        self.focus_id = Some(id.into());
        self.soft_kb_open = true;
        self.soft_kb_mode = mode;
    }

    pub fn clear_focus(&mut self) {
        self.focus_id = None;
        self.soft_kb_open = false;
    }

    pub fn spring(
        &mut self,
        id: impl Into<WidgetId>,
        channel: u8,
        initial: f32,
        spatial: bool,
    ) -> f32 {
        let key = (id.into(), channel);
        let spec = self.theme.motion_scheme().spec(
            Tempo::Default,
            if spatial {
                Track::Spatial
            } else {
                Track::Effects
            },
        );
        self.springs
            .entry(key)
            .or_insert_with(|| DynamicSpring::new(initial, initial, spec))
            .x
    }

    pub fn spring_retarget(
        &mut self,
        id: impl Into<WidgetId>,
        channel: u8,
        target: f32,
        tempo: Tempo,
        track: Track,
    ) -> f32 {
        let key = (id.into(), channel);
        let spec = self.theme.motion_scheme().spec(tempo, track);
        let s = self
            .springs
            .entry(key)
            .or_insert_with(|| DynamicSpring::new(target, target, spec));
        s.spec = spec;
        if (s.target - target).abs() > 1e-4 {
            s.retarget(target);
        }
        s.x
    }

    pub fn spring_kick(
        &mut self,
        id: impl Into<WidgetId>,
        channel: u8,
        pos: f32,
        target: f32,
        tempo: Tempo,
        track: Track,
    ) {
        let key = (id.into(), channel);
        let spec = self.theme.motion_scheme().spec(tempo, track);
        let s = self
            .springs
            .entry(key)
            .or_insert_with(|| DynamicSpring::new(pos, target, spec));
        s.spec = spec;
        s.x = pos;
        s.v = 0.0;
        s.target = target;
    }

    pub fn step(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        self.t += dt;
        for s in self.springs.values_mut() {
            s.step(dt);
        }
        let now = self.t;
        self.ripples.retain(|r| match r.release_t {
            Some(rt) => now - rt < 0.42,
            None => true,
        });
    }

    pub fn scroll_by(&mut self, dy: f32) {
        let u = self.u();
        let kb_extra = if self.soft_kb_open { 232.0 * u } else { 0.0 };
        let max_scroll = (self.content_h + kb_extra - self.height).max(0.0);
        self.scroll = (self.scroll + dy).clamp(0.0, max_scroll);
    }

    pub fn push_key(&mut self, key: KeyInput) {
        self.key_queue.push(key);
    }

    pub fn has_hit(&self, x: f32, y: f32) -> bool {
        self.hit_test(x, y).is_some()
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<HitZone> {
        let u = self.u();
        let top_bar_bottom = 102.0 * u;
        let kb_top = if self.soft_kb_open {
            self.height - 224.0 * u
        } else {
            self.height + 1000.0
        };

        // 1. Check overlay zones first (e.g. soft keyboard, app bar, open menus)
        for z in self.zones.iter().rev() {
            if z.overlay && z.rect.contains(x, y) {
                return Some(*z);
            }
        }
        if y < top_bar_bottom || y >= kb_top {
            return None;
        }
        // 2. Check regular scrollable content zones
        for z in self.zones.iter().rev() {
            if !z.overlay && z.rect.contains(x, y) {
                return Some(*z);
            }
        }
        None
    }

    pub fn pointer_down(&mut self, x: f32, y: f32) {
        self.pointer = (x, y);
        self.press_start = (x, y);
        self.prev_pointer = (x, y);
        self.frame_delta = (0.0, 0.0);
        self.is_down = true;
        self.dragging_scroll = false;
        self.active_drag_id = None;
        self.just_clicked = None;

        let hit = self.hit_test(x, y);
        self.active_zone = hit;

        if let Some(z) = hit {
            match z.kind {
                ZoneKind::ImmediateHorizontalDrag | ZoneKind::DialDrag { .. } => {
                    self.active_drag_id = Some(z.id);
                }
                _ => {}
            }
            let bounded = match z.kind {
                ZoneKind::Tap { bounded } => bounded,
                _ => true,
            };
            self.ripples.push(Ripple {
                id: z.id,
                cx: x,
                cy: y,
                rect: z.rect,
                color: z.ripple_color,
                start_t: self.t,
                release_t: None,
                bounded,
            });
        }
    }

    pub fn pointer_move(&mut self, x: f32, y: f32) {
        let dx = x - self.prev_pointer.0;
        let dy = y - self.prev_pointer.1;
        self.prev_pointer = (x, y);
        self.pointer = (x, y);
        if !self.is_down {
            return;
        }
        self.frame_delta.0 += dx;
        self.frame_delta.1 += dy;

        let total_dx = x - self.press_start.0;
        let total_dy = y - self.press_start.1;
        let u = self.u();
        let slop = 8.0 * u;

        if let Some(z) = self.active_zone {
            match z.kind {
                ZoneKind::ImmediateHorizontalDrag | ZoneKind::DialDrag { .. } => {
                    self.active_drag_id = Some(z.id);
                    return;
                }
                ZoneKind::SwipeHorizontalDrag => {
                    if self.active_drag_id == Some(z.id) {
                        return;
                    }
                    if total_dx.abs() > slop && total_dx.abs() >= total_dy.abs() * 0.85 {
                        self.active_drag_id = Some(z.id);
                        return;
                    }
                    if !z.overlay && total_dy.abs() > slop * 1.2 && total_dy.abs() > total_dx.abs() {
                        self.dragging_scroll = true;
                        self.active_zone = None;
                        self.release_active_ripples();
                    }
                }
                ZoneKind::VerticalPullDrag => {
                    if self.active_drag_id == Some(z.id) {
                        return;
                    }
                    if total_dy > slop && total_dy >= total_dx.abs() {
                        self.active_drag_id = Some(z.id);
                        return;
                    }
                    if total_dy < -slop {
                        self.dragging_scroll = true;
                        self.active_zone = None;
                        self.release_active_ripples();
                    }
                }
                ZoneKind::Tap { .. } => {
                    // Check if this tap started inside a horizontal swipe container (e.g. tabs or chips)
                    if total_dx.abs() > slop && total_dx.abs() >= total_dy.abs() * 0.85 {
                        if let Some(swipe_zone) = self.zones.iter().rev().find(|sz| {
                            sz.kind == ZoneKind::SwipeHorizontalDrag
                                && sz.rect.contains(self.press_start.0, self.press_start.1)
                        }) {
                            self.active_zone = Some(*swipe_zone);
                            self.active_drag_id = Some(swipe_zone.id);
                            self.release_active_ripples();
                            return;
                        }
                    }
                    if total_dx.hypot(total_dy) > slop {
                        if !z.overlay {
                            self.dragging_scroll = true;
                        }
                        self.active_zone = None;
                        self.release_active_ripples();
                    }
                }
            }
        } else if total_dx.hypot(total_dy) > slop * 0.7 {
            let kb_top = if self.soft_kb_open {
                self.height - 224.0 * u
            } else {
                self.height + 1000.0
            };
            if self.press_start.1 < kb_top {
                self.dragging_scroll = true;
            }
        }

        if self.dragging_scroll {
            self.scroll_by(-dy);
        }
    }

    pub fn pointer_up(&mut self, x: f32, y: f32) {
        self.pointer = (x, y);
        self.release_active_ripples();

        if self.active_drag_id.is_none() && !self.dragging_scroll {
            if let Some(z) = self.active_zone {
                let u = self.u();
                if z.rect.contains(x, y)
                    || (x - self.press_start.0).hypot(y - self.press_start.1) < 12.0 * u
                {
                    self.just_clicked = Some(z.id);
                }
            } else {
                let u = self.u();
                let kb_top = if self.soft_kb_open {
                    self.height - 224.0 * u
                } else {
                    self.height + 1000.0
                };
                if y < kb_top
                    && (x - self.press_start.0).hypot(y - self.press_start.1) < 10.0 * u
                {
                    self.clear_focus();
                }
            }
        }

        self.is_down = false;
        self.dragging_scroll = false;
        self.active_zone = None;
        self.active_drag_id = None;
    }

    fn release_active_ripples(&mut self) {
        let now = self.t;
        for r in &mut self.ripples {
            if r.release_t.is_none() {
                r.release_t = Some(now);
            }
        }
    }
}

pub struct UiContext<'a> {
    pub renderer: &'a mut UiRenderer,
    pub state: &'a mut UiState,
    pub frame_delta: (f32, f32),
    pub in_overlay: bool,
}

impl<'a> UiContext<'a> {
    pub fn begin(
        renderer: &'a mut UiRenderer,
        state: &'a mut UiState,
        width: f32,
        height: f32,
    ) -> Self {
        state.width = width.max(1.0);
        state.height = height.max(1.0);
        state.zones.clear();
        let frame_delta = state.frame_delta;
        state.frame_delta = (0.0, 0.0);
        Self {
            renderer,
            state,
            frame_delta,
            in_overlay: false,
        }
    }

    pub fn u(&self) -> f32 {
        self.state.u()
    }

    pub fn pointer_pos(&self) -> (f32, f32) {
        self.state.pointer
    }

    pub fn is_pressed(&self, id: impl Into<WidgetId>) -> bool {
        let id = id.into();
        self.state.is_down
            && !self.state.dragging_scroll
            && self.state.active_zone.map(|z| z.id) == Some(id)
    }

    pub fn is_focused(&self, id: impl Into<WidgetId>) -> bool {
        self.state.focus_id == Some(id.into())
    }

    pub fn elev_dp(&mut self, id: impl Into<WidgetId>) -> f32 {
        let id = id.into();
        let pressed = self.is_pressed(id);
        let target = if pressed { 2.0 } else { 6.0 };
        self.state
            .spring_retarget(id, 30, target, Tempo::Fast, Track::Effects)
            .clamp(0.0, 12.0)
    }

    pub fn draw_state_layer(
        &mut self,
        id: impl Into<WidgetId>,
        rect: Rect,
        radii: [f32; 4],
        ink: [f32; 4],
    ) {
        let id = id.into();
        let pressed = self.is_pressed(id);
        let a = self
            .state
            .spring_retarget(
                id,
                31,
                if pressed { 0.12 } else { 0.0 },
                Tempo::Fast,
                Track::Effects,
            )
            .clamp(0.0, 0.25);
        if a > 0.004 {
            self.renderer
                .rect(rect.x, rect.y, rect.w, rect.h, with_alpha(ink, a), radii);
        }
    }

    pub fn draw_state_circle(
        &mut self,
        id: impl Into<WidgetId>,
        cx: f32,
        cy: f32,
        radius: f32,
        ink: [f32; 4],
    ) {
        let id = id.into();
        let pressed = self.is_pressed(id);
        let a = self
            .state
            .spring_retarget(
                id,
                31,
                if pressed { 0.12 } else { 0.0 },
                Tempo::Fast,
                Track::Effects,
            )
            .clamp(0.0, 0.25);
        if a > 0.004 {
            self.renderer.circle(cx, cy, radius, with_alpha(ink, a));
        }
    }

    pub fn draw_ripples(&mut self, id: impl Into<WidgetId>, fallback_rect: Rect) {
        let id = id.into();
        let now = self.state.t;
        for r in &self.state.ripples {
            if r.id != id {
                continue;
            }
            let age = (now - r.start_t).max(0.0);
            let expand = (age / 0.28).clamp(0.0, 1.0);
            let fade = match r.release_t {
                Some(rt) => (1.0 - ((now - rt) / 0.36).clamp(0.0, 1.0)).powi(2),
                None => 1.0,
            };
            let alpha = 0.16 * fade;
            if alpha <= 0.004 {
                continue;
            }
            let bounds = if r.rect.w > 0.0 {
                r.rect
            } else {
                fallback_rect
            };
            let max_r = bounds.w.hypot(bounds.h) * 0.65;
            let cur_r = lerp(bounds.w.min(bounds.h) * 0.15, max_r, expand);
            if r.bounded {
                self.renderer
                    .push_clip([bounds.x, bounds.y, bounds.w, bounds.h]);
                self.renderer
                    .circle(r.cx, r.cy, cur_r, with_alpha(r.color, alpha));
                self.renderer.pop_clip();
            } else {
                self.renderer
                    .circle(r.cx, r.cy, cur_r.min(28.0 * self.u()), with_alpha(r.color, alpha));
            }
        }
    }

    pub fn register_zone(
        &mut self,
        id: impl Into<WidgetId>,
        rect: Rect,
        ripple_color: [f32; 4],
        kind: ZoneKind,
        overlay: bool,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let is_app_bar_item = rect.y < 102.0 * u;
        self.state.zones.push(HitZone {
            id,
            rect,
            ripple_color,
            kind,
            overlay: overlay || self.in_overlay || is_app_bar_item,
        });

        let clicked = self.state.just_clicked == Some(id);
        if clicked {
            self.state.just_clicked = None;
        }
        let pressed = self.is_pressed(id);
        let dragged = self.state.active_drag_id == Some(id);
        let focused = self.is_focused(id);
        let (dx, dy) = if dragged {
            self.frame_delta
        } else {
            (0.0, 0.0)
        };

        WidgetResponse {
            clicked,
            pressed,
            dragged,
            focused,
            changed: false,
            drag_delta_x: dx,
            drag_delta_y: dy,
            rect,
        }
    }

    pub fn apply_keys_to_field(&mut self, id: WidgetId, props: &mut TextFieldProps) -> bool {
        if self.state.focus_id != Some(id) || self.state.key_queue.is_empty() {
            return false;
        }
        let keys: Vec<KeyInput> = std::mem::take(&mut self.state.key_queue);
        let mut changed = false;
        for k in keys {
            match k {
                KeyInput::Char(ch) => {
                    if props.insert_char(ch) {
                        changed = true;
                    }
                }
                KeyInput::Text(s) => {
                    if props.insert_str(&s) {
                        changed = true;
                    }
                }
                KeyInput::Backspace => {
                    if props.backspace() {
                        changed = true;
                    }
                }
                KeyInput::Delete => {
                    if props.delete_forward() {
                        changed = true;
                    }
                }
                KeyInput::Left => props.move_left(),
                KeyInput::Right => props.move_right(),
                KeyInput::Home => props.move_home(),
                KeyInput::End => props.move_end(),
                KeyInput::Enter | KeyInput::Escape => {
                    self.state.clear_focus();
                }
            }
        }
        changed
    }

    /// Completes the frame and renders the docked M3 Expressive SoftKeyboard when a text or numeric field is focused.
    pub fn finish_frame(&mut self) {
        let u = self.u();
        let kb_anim = self
            .state
            .spring_retarget(
                WidgetId::from_label("soft_keyboard_sheet"),
                0,
                f32::from(self.state.soft_kb_open),
                Tempo::Fast,
                Track::Spatial,
            )
            .clamp(0.0, 1.0);
        if kb_anim < 0.02 && !self.state.soft_kb_open {
            return;
        }
        let kb_h = 224.0 * u;
        let width = self.state.width;
        let y_top = self.state.height - kb_h * kb_anim;
        let pal = self.state.pal.clone();

        // Backdrop zone so touches on keyboard don't click underlying cards
        let _ = self.register_zone(
            WidgetId::from_label("soft_kb_backdrop"),
            Rect::new(0.0, y_top, width, kb_h),
            pal.text,
            ZoneKind::Tap { bounded: true },
            true,
        );

        self.renderer.rect_styled(
            0.0,
            y_top,
            width,
            kb_h + 16.0 * u,
            pal.card_hi,
            pal.outline_variant,
            1.0 * u,
            [24.0 * u, 24.0 * u, 0.0, 0.0],
            0.0,
        );

        // Top strip: Mode pills ("ABC", "123 / Date", "#+=") + Done button
        let strip_y = y_top + 8.0 * u;
        let mode_specs = [
            ("kb_m_abc", "ABC", SoftKeyboardMode::Qwerty, false),
            ("kb_m_num", "123 / Date", SoftKeyboardMode::Numeric, false),
            ("kb_m_sym", "#+=", SoftKeyboardMode::Qwerty, true),
        ];
        let mut mx = 12.0 * u;
        for (mid_str, lbl, m, sym) in mode_specs {
            let active = self.state.soft_kb_mode == m
                && (m == SoftKeyboardMode::Numeric || self.state.soft_kb_symbols == sym);
            let mw = if m == SoftKeyboardMode::Numeric {
                88.0 * u
            } else {
                54.0 * u
            };
            let mr = Rect::new(mx, strip_y, mw, 28.0 * u);
            let mid = WidgetId::from_label(mid_str);
            let fill = if active {
                pal.secondary_container
            } else {
                pal.card_highest
            };
            let ink = if active {
                pal.on_secondary_container
            } else {
                pal.sub
            };
            self.renderer
                .rect(mr.x, mr.y, mr.w, mr.h, fill, r4(14.0 * u));
            text_c(
                self.renderer,
                lbl,
                mr.x + mr.w * 0.5,
                mr.y + 6.0 * u,
                ts(M3_LABEL_MEDIUM, ink, u),
            );
            if self
                .register_zone(mid, mr, ink, ZoneKind::Tap { bounded: true }, true)
                .clicked
            {
                self.state.soft_kb_mode = m;
                self.state.soft_kb_symbols = sym;
            }
            mx += mw + 6.0 * u;
        }

        // Done button on right of strip
        let done_w = 68.0 * u;
        let done_r = Rect::new(width - done_w - 12.0 * u, strip_y, done_w, 28.0 * u);
        let done_id = WidgetId::from_label("kb_done_btn");
        self.renderer
            .rect(done_r.x, done_r.y, done_r.w, done_r.h, pal.primary, r4(14.0 * u));
        text_c(
            self.renderer,
            "Done",
            done_r.x + done_r.w * 0.5,
            done_r.y + 6.0 * u,
            ts(M3_LABEL_MEDIUM, pal.on_primary, u),
        );
        if self
            .register_zone(
                done_id,
                done_r,
                pal.on_primary,
                ZoneKind::Tap { bounded: true },
                true,
            )
            .clicked
        {
            self.state.push_key(KeyInput::Enter);
        }

        let keys_top = strip_y + 34.0 * u;
        match self.state.soft_kb_mode {
            SoftKeyboardMode::Numeric => {
                let rows: [[&str; 4]; 4] = [
                    ["1", "2", "3", "/"],
                    ["4", "5", "6", "-"],
                    ["7", "8", "9", "."],
                    ["CLR", "0", "⌫", "OK"],
                ];
                let pad_x = 12.0 * u;
                let gap = 6.0 * u;
                let row_h = 37.0 * u;
                let col_w = (width - pad_x * 2.0 - gap * 3.0) / 4.0;
                for (ri, row) in rows.iter().enumerate() {
                    let ry = keys_top + ri as f32 * (row_h + 6.0 * u);
                    for (ci, k) in row.iter().enumerate() {
                        let kx = pad_x + ci as f32 * (col_w + gap);
                        let kr = Rect::new(kx, ry, col_w, row_h);
                        let kid = WidgetId::from_label("kb_num_key").with_index(ri * 10 + ci);
                        let is_ok = *k == "OK";
                        let is_act = matches!(*k, "/" | "-" | "." | "CLR" | "⌫");
                        let fill = if is_ok {
                            pal.primary
                        } else if is_act {
                            pal.secondary_container
                        } else {
                            pal.card_highest
                        };
                        let ink = if is_ok {
                            pal.on_primary
                        } else if is_act {
                            pal.on_secondary_container
                        } else {
                            pal.text
                        };
                        self.renderer
                            .rect(kr.x, kr.y, kr.w, kr.h, fill, r4(12.0 * u));
                        self.draw_state_layer(kid, kr, r4(12.0 * u), ink);
                        self.draw_ripples(kid, kr);
                        text_c(
                            self.renderer,
                            k,
                            kr.x + kr.w * 0.5,
                            kr.y + 8.0 * u,
                            ts(M3_TITLE_MEDIUM, ink, u),
                        );
                        if self
                            .register_zone(kid, kr, ink, ZoneKind::Tap { bounded: true }, true)
                            .clicked
                        {
                            match *k {
                                "⌫" => self.state.push_key(KeyInput::Backspace),
                                "CLR" => {
                                    for _ in 0..24 {
                                        self.state.push_key(KeyInput::Backspace);
                                    }
                                }
                                "OK" => self.state.push_key(KeyInput::Enter),
                                s => {
                                    if let Some(ch) = s.chars().next() {
                                        self.state.push_key(KeyInput::Char(ch));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            SoftKeyboardMode::Qwerty => {
                let sym = self.state.soft_kb_symbols;
                let up = self.state.soft_kb_shift;
                let r0: [&str; 10] = if sym {
                    ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"]
                } else if up {
                    ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"]
                } else {
                    ["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"]
                };
                let r1: [&str; 9] = if sym {
                    ["@", "#", "$", "_", "&", "-", "+", "(", ")"]
                } else if up {
                    ["A", "S", "D", "F", "G", "H", "J", "K", "L"]
                } else {
                    ["a", "s", "d", "f", "g", "h", "j", "k", "l"]
                };
                let r2: [&str; 7] = if sym {
                    ["*", "\"", "'", ":", ";", "!", "?"]
                } else if up {
                    ["Z", "X", "C", "V", "B", "N", "M"]
                } else {
                    ["z", "x", "c", "v", "b", "n", "m"]
                };

                let pad_x = 6.0 * u;
                let gap = 4.0 * u;
                let row_h = 37.0 * u;
                let key_w = (width - pad_x * 2.0 - gap * 9.0) / 10.0;

                for (i, k) in r0.iter().enumerate() {
                    let kx = pad_x + i as f32 * (key_w + gap);
                    let kr = Rect::new(kx, keys_top, key_w, row_h);
                    let kid = WidgetId::from_label("kb_q0").with_index(i);
                    self.renderer
                        .rect(kr.x, kr.y, kr.w, kr.h, pal.card_highest, r4(8.0 * u));
                    self.draw_state_layer(kid, kr, r4(8.0 * u), pal.text);
                    self.draw_ripples(kid, kr);
                    text_c(
                        self.renderer,
                        k,
                        kr.x + kr.w * 0.5,
                        kr.y + 9.0 * u,
                        ts(M3_LABEL_LARGE, pal.text, u),
                    );
                    if self
                        .register_zone(kid, kr, pal.text, ZoneKind::Tap { bounded: true }, true)
                        .clicked
                    {
                        if let Some(ch) = k.chars().next() {
                            self.state.push_key(KeyInput::Char(ch));
                        }
                    }
                }

                let r1_y = keys_top + row_h + 6.0 * u;
                let r1_w = 9.0 * key_w + 8.0 * gap;
                let r1_x0 = (width - r1_w) * 0.5;
                for (i, k) in r1.iter().enumerate() {
                    let kx = r1_x0 + i as f32 * (key_w + gap);
                    let kr = Rect::new(kx, r1_y, key_w, row_h);
                    let kid = WidgetId::from_label("kb_q1").with_index(i);
                    self.renderer
                        .rect(kr.x, kr.y, kr.w, kr.h, pal.card_highest, r4(8.0 * u));
                    self.draw_state_layer(kid, kr, r4(8.0 * u), pal.text);
                    self.draw_ripples(kid, kr);
                    text_c(
                        self.renderer,
                        k,
                        kr.x + kr.w * 0.5,
                        kr.y + 9.0 * u,
                        ts(M3_LABEL_LARGE, pal.text, u),
                    );
                    if self
                        .register_zone(kid, kr, pal.text, ZoneKind::Tap { bounded: true }, true)
                        .clicked
                    {
                        if let Some(ch) = k.chars().next() {
                            self.state.push_key(KeyInput::Char(ch));
                        }
                    }
                }

                let r2_y = r1_y + row_h + 6.0 * u;
                let side_w = key_w * 1.42;
                let shift_r = Rect::new(pad_x, r2_y, side_w, row_h);
                let shift_id = WidgetId::from_label("kb_shift_key");
                let sh_fill = if !sym && up {
                    pal.primary
                } else {
                    pal.secondary_container
                };
                let sh_ink = if !sym && up {
                    pal.on_primary
                } else {
                    pal.on_secondary_container
                };
                self.renderer.rect(
                    shift_r.x,
                    shift_r.y,
                    shift_r.w,
                    shift_r.h,
                    sh_fill,
                    r4(10.0 * u),
                );
                text_c(
                    self.renderer,
                    if sym { "ABC" } else { "⇧" },
                    shift_r.x + shift_r.w * 0.5,
                    shift_r.y + 9.0 * u,
                    ts(M3_LABEL_LARGE, sh_ink, u),
                );
                if self
                    .register_zone(
                        shift_id,
                        shift_r,
                        sh_ink,
                        ZoneKind::Tap { bounded: true },
                        true,
                    )
                    .clicked
                {
                    if sym {
                        self.state.soft_kb_symbols = false;
                    } else {
                        self.state.soft_kb_shift = !self.state.soft_kb_shift;
                    }
                }

                let r2_w = 7.0 * key_w + 6.0 * gap;
                let r2_x0 = (width - r2_w) * 0.5;
                for (i, k) in r2.iter().enumerate() {
                    let kx = r2_x0 + i as f32 * (key_w + gap);
                    let kr = Rect::new(kx, r2_y, key_w, row_h);
                    let kid = WidgetId::from_label("kb_q2").with_index(i);
                    self.renderer
                        .rect(kr.x, kr.y, kr.w, kr.h, pal.card_highest, r4(8.0 * u));
                    self.draw_state_layer(kid, kr, r4(8.0 * u), pal.text);
                    self.draw_ripples(kid, kr);
                    text_c(
                        self.renderer,
                        k,
                        kr.x + kr.w * 0.5,
                        kr.y + 9.0 * u,
                        ts(M3_LABEL_LARGE, pal.text, u),
                    );
                    if self
                        .register_zone(kid, kr, pal.text, ZoneKind::Tap { bounded: true }, true)
                        .clicked
                    {
                        if let Some(ch) = k.chars().next() {
                            self.state.push_key(KeyInput::Char(ch));
                        }
                    }
                }

                let bs_r = Rect::new(width - pad_x - side_w, r2_y, side_w, row_h);
                let bs_id = WidgetId::from_label("kb_bs_key");
                self.renderer.rect(
                    bs_r.x,
                    bs_r.y,
                    bs_r.w,
                    bs_r.h,
                    pal.secondary_container,
                    r4(10.0 * u),
                );
                icon_c(
                    self.renderer,
                    "backspace",
                    bs_r.x + bs_r.w * 0.5,
                    bs_r.y + bs_r.h * 0.5,
                    18.0 * u,
                    pal.on_secondary_container,
                );
                if self
                    .register_zone(
                        bs_id,
                        bs_r,
                        pal.on_secondary_container,
                        ZoneKind::Tap { bounded: true },
                        true,
                    )
                    .clicked
                {
                    self.state.push_key(KeyInput::Backspace);
                }

                // Bottom row: 123, '@', '/', Space, '.', Enter
                let r3_y = r2_y + row_h + 6.0 * u;
                let num_w = 50.0 * u;
                let num_r = Rect::new(pad_x, r3_y, num_w, row_h);
                let num_id = WidgetId::from_label("kb_to_123");
                self.renderer.rect(
                    num_r.x,
                    num_r.y,
                    num_r.w,
                    num_r.h,
                    pal.secondary_container,
                    r4(10.0 * u),
                );
                text_c(
                    self.renderer,
                    "123",
                    num_r.x + num_r.w * 0.5,
                    num_r.y + 9.0 * u,
                    ts(M3_LABEL_LARGE, pal.on_secondary_container, u),
                );
                if self
                    .register_zone(
                        num_id,
                        num_r,
                        pal.on_secondary_container,
                        ZoneKind::Tap { bounded: true },
                        true,
                    )
                    .clicked
                {
                    self.state.soft_kb_mode = SoftKeyboardMode::Numeric;
                }

                let at_w = 34.0 * u;
                let at_r = Rect::new(num_r.x + num_r.w + gap, r3_y, at_w, row_h);
                let at_id = WidgetId::from_label("kb_at");
                self.renderer
                    .rect(at_r.x, at_r.y, at_r.w, at_r.h, pal.card_highest, r4(8.0 * u));
                text_c(
                    self.renderer,
                    "@",
                    at_r.x + at_r.w * 0.5,
                    at_r.y + 9.0 * u,
                    ts(M3_LABEL_LARGE, pal.text, u),
                );
                if self
                    .register_zone(at_id, at_r, pal.text, ZoneKind::Tap { bounded: true }, true)
                    .clicked
                {
                    self.state.push_key(KeyInput::Char('@'));
                }

                let enter_w = 62.0 * u;
                let dot_w = 34.0 * u;
                let sp_x = at_r.x + at_r.w + gap;
                let sp_w = (width - pad_x - enter_w - gap - dot_w - gap - sp_x).max(80.0 * u);
                let sp_r = Rect::new(sp_x, r3_y, sp_w, row_h);
                let sp_id = WidgetId::from_label("kb_space");
                self.renderer
                    .rect(sp_r.x, sp_r.y, sp_r.w, sp_r.h, pal.card_highest, r4(10.0 * u));
                text_c(
                    self.renderer,
                    "Space",
                    sp_r.x + sp_r.w * 0.5,
                    sp_r.y + 10.0 * u,
                    ts(M3_LABEL_MEDIUM, pal.sub, u),
                );
                if self
                    .register_zone(sp_id, sp_r, pal.text, ZoneKind::Tap { bounded: true }, true)
                    .clicked
                {
                    self.state.push_key(KeyInput::Char(' '));
                }

                let dot_r = Rect::new(sp_r.x + sp_r.w + gap, r3_y, dot_w, row_h);
                let dot_id = WidgetId::from_label("kb_dot");
                self.renderer.rect(
                    dot_r.x,
                    dot_r.y,
                    dot_r.w,
                    dot_r.h,
                    pal.card_highest,
                    r4(8.0 * u),
                );
                text_c(
                    self.renderer,
                    ".",
                    dot_r.x + dot_r.w * 0.5,
                    dot_r.y + 9.0 * u,
                    ts(M3_LABEL_LARGE, pal.text, u),
                );
                if self
                    .register_zone(
                        dot_id,
                        dot_r,
                        pal.text,
                        ZoneKind::Tap { bounded: true },
                        true,
                    )
                    .clicked
                {
                    self.state.push_key(KeyInput::Char('.'));
                }

                let ent_r = Rect::new(width - pad_x - enter_w, r3_y, enter_w, row_h);
                let ent_id = WidgetId::from_label("kb_enter");
                self.renderer
                    .rect(ent_r.x, ent_r.y, ent_r.w, ent_r.h, pal.primary, r4(12.0 * u));
                icon_c(
                    self.renderer,
                    "keyboard_return",
                    ent_r.x + ent_r.w * 0.5,
                    ent_r.y + ent_r.h * 0.5,
                    20.0 * u,
                    pal.on_primary,
                );
                if self
                    .register_zone(
                        ent_id,
                        ent_r,
                        pal.on_primary,
                        ZoneKind::Tap { bounded: true },
                        true,
                    )
                    .clicked
                {
                    self.state.push_key(KeyInput::Enter);
                }
            }
        }
    }
}

/// Vertical stack layout helper for declarative-style UI composition.
#[derive(Clone, Copy, Debug)]
pub struct ColumnLayout {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub spacing: f32,
}

impl ColumnLayout {
    pub const fn new(x: f32, y: f32, w: f32, spacing: f32) -> Self {
        Self { x, y, w, spacing }
    }

    pub fn next(&mut self, h: f32) -> Rect {
        let r = Rect::new(self.x, self.y, self.w, h);
        self.y += h + self.spacing;
        r
    }

    pub fn gap(&mut self, dy: f32) {
        self.y += dy;
    }
}

/// Horizontal row layout helper for declarative-style UI composition.
#[derive(Clone, Copy, Debug)]
pub struct RowLayout {
    pub x: f32,
    pub y: f32,
    pub h: f32,
    pub spacing: f32,
}

impl RowLayout {
    pub const fn new(x: f32, y: f32, h: f32, spacing: f32) -> Self {
        Self { x, y, h, spacing }
    }

    pub fn next(&mut self, w: f32) -> Rect {
        let r = Rect::new(self.x, self.y, w, self.h);
        self.x += w + self.spacing;
        r
    }
}
