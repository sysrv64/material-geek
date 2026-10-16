use mg_components::ComponentId;
use mg_motion::{cubic_bezier_y, DynamicSpring, MotionScheme, Tempo, Track};
use mg_render::{
    m3_text_style, MaterialShape, TextStyle, TypeStyle, UiRenderer, M3_BODY_LARGE, M3_BODY_MEDIUM,
    M3_BODY_SMALL, M3_DISPLAY_SMALL, M3_HEADLINE_SMALL, M3_LABEL_LARGE, M3_LABEL_MEDIUM,
    M3_LABEL_SMALL, M3_TITLE_LARGE, M3_TITLE_MEDIUM, M3_TITLE_SMALL,
};
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::sync::Arc;
use std::time::Instant;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

fn srgb(byte: u8) -> f32 {
    byte as f32 / 255.0
}

fn argb(argb: u32) -> [f32; 4] {
    [
        srgb(((argb >> 16) & 0xFF) as u8),
        srgb(((argb >> 8) & 0xFF) as u8),
        srgb((argb & 0xFF) as u8),
        1.0,
    ]
}

fn r4(r: f32) -> [f32; 4] {
    [r, r, r, r]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn with_alpha(c: [f32; 4], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], c[3] * a]
}

fn mix(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

fn ts(t: TypeStyle, color: [f32; 4], u: f32) -> TextStyle {
    let mut s = m3_text_style(t, color);
    s.px *= u;
    s.line_h *= u;
    s
}

#[derive(Clone, Copy, PartialEq)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }
}

fn icon_c(ui: &mut UiRenderer, name: &str, cx: f32, cy: f32, px: f32, color: [f32; 4]) {
    ui.icon(name, cx - px * 0.5, cy - px * 0.5, px, color);
}

fn text_c(ui: &mut UiRenderer, s: &str, cx: f32, y_top: f32, style: TextStyle) {
    let tw = ui.text_width(s, style.px, style.medium, style.tracking_em);
    ui.text(s, cx - tw * 0.5, y_top, style);
}

/// Evaluates the M3 Expressive 7-shape loading sequence (`SoftBurst -> Cookie9Sided -> Pentagon -> Pill -> Sunny -> Cookie4Sided -> Oval`)
/// returning `(shape_a, shape_b, morph_t, rotation_rad, scale_factor)`.
fn loading_morph_state(
    t: f32,
    phase_offset: usize,
) -> (MaterialShape, MaterialShape, f32, f32, f32) {
    let seq = &MaterialShape::LOADING_SEQUENCE;
    let cycle_dur = 0.65;
    let total = t / cycle_dur + phase_offset as f32;
    let step = total.floor() as usize;
    let local = total.fract();
    // Expressive spring-like fast morph during the first 68% of each step, with subtle overshoot settle
    let morph_t = if local < 0.68 {
        let u = local / 0.68;
        cubic_bezier_y(u, 0.22, 1.0, 0.36, 1.0)
    } else {
        1.0
    };
    let shape_a = seq[step % seq.len()];
    let shape_b = seq[(step + 1) % seq.len()];
    // Continuous global rotation (4.666s period) + 90deg step progression
    let rot = t * (TAU / 4.666) + (step as f32 + morph_t) * FRAC_PI_2;
    let scale = 1.0 - 0.06 * (morph_t * PI).sin();
    (shape_a, shape_b, morph_t, rot, scale)
}

#[derive(Clone, Copy, PartialEq)]
enum Action {
    Switch(usize),
    Check(usize),
    Radio(usize),
    Chip(usize),
    Tab(usize),
    SecondaryTab(usize),
    Nav(usize),
    Rail(usize),
    Drawer(usize),
    Group(usize),
    StdGroup(usize),
    Segmented(usize),
    MenuItem(usize),
    CarouselPage(usize),
    CarouselMode(usize),
    DateDay(usize),
    TimeAmPm(bool),
    ToggleBtn(usize),
    IconToggle(usize),
    ExtendedFabToggle,
    SplitTrailing(usize),
    FabMenu,
    Button(u32),
    Slider,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Btn {
    Filled,
    Tonal,
    Outlined,
    Elevated,
    Text,
}

struct BtnSpec<'a> {
    label: &'a str,
    icon: Option<&'a str>,
    variant: Btn,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IconBtn {
    Standard,
    Filled,
    Tonal,
    Outlined,
}

struct Palette {
    bg: [f32; 4],
    surface_low: [f32; 4],
    card: [f32; 4],
    card_hi: [f32; 4],
    card_highest: [f32; 4],
    text: [f32; 4],
    sub: [f32; 4],
    primary: [f32; 4],
    on_primary: [f32; 4],
    primary_container: [f32; 4],
    on_primary_container: [f32; 4],
    secondary: [f32; 4],
    secondary_container: [f32; 4],
    on_secondary_container: [f32; 4],
    tertiary: [f32; 4],
    tertiary_container: [f32; 4],
    on_tertiary_container: [f32; 4],
    error: [f32; 4],
    on_error: [f32; 4],
    error_container: [f32; 4],
    on_error_container: [f32; 4],
    outline: [f32; 4],
    outline_variant: [f32; 4],
    inverse: [f32; 4],
    on_inverse: [f32; 4],
    inverse_primary: [f32; 4],
}

impl Palette {
    fn dark() -> Self {
        Self {
            bg: argb(0x141218),
            surface_low: argb(0x1D1B20),
            card: argb(0x211F26),
            card_hi: argb(0x2B2930),
            card_highest: argb(0x36343B),
            text: argb(0xE6E0E9),
            sub: argb(0xCAC4D0),
            primary: argb(0xD0BCFF),
            on_primary: argb(0x381E72),
            primary_container: argb(0x4F378B),
            on_primary_container: argb(0xEADDFF),
            secondary: argb(0xCCC2DC),
            secondary_container: argb(0x4A4458),
            on_secondary_container: argb(0xE8DEF8),
            tertiary: argb(0xEFB8C8),
            tertiary_container: argb(0x633B48),
            on_tertiary_container: argb(0xFFD8E4),
            error: argb(0xF2B8B5),
            on_error: argb(0x601410),
            error_container: argb(0x8C1D18),
            on_error_container: argb(0xF9DEDC),
            outline: argb(0x938F99),
            outline_variant: argb(0x49454F),
            inverse: argb(0xE6E0E9),
            on_inverse: argb(0x313033),
            inverse_primary: argb(0x6750A4),
        }
    }
}

#[derive(Clone, Copy)]
struct Pressed {
    action: Action,
    start: f32,
}

struct Ripple {
    rect: Rect,
    x: f32,
    y: f32,
    start: f32,
    released: Option<f32>,
    bounded: bool,
    ink: [f32; 4],
}

pub struct Gallery {
    pal: Palette,
    u: f32,
    scroll: f32,
    content_h: f32,
    view_h: f32,
    down: Option<(f32, f32)>,
    down_moved: bool,
    drag_last_y: f32,
    slider_drag: bool,
    pressed: Option<Pressed>,
    last_release: Option<(Action, f32)>,
    ripples: Vec<Ripple>,
    start: Instant,
    t: f32,
    morph: DynamicSpring,
    morph_key: u32,
    group_press: DynamicSpring,
    group_pressed_idx: Option<usize>,
    slider_thumb: DynamicSpring,
    slider: f32,
    focus: DynamicSpring,
    switches: [bool; 3],
    switch_anim: [DynamicSpring; 3],
    check_state: [u8; 3],
    check_draw: [DynamicSpring; 3],
    check_color: [DynamicSpring; 3],
    check_uncheck_at: [f32; 3],
    radio: usize,
    radio_anim: [DynamicSpring; 3],
    chips: [bool; 4],
    chip_anim: [DynamicSpring; 4],
    toggle_btns: [bool; 3],
    toggle_btn_anim: [DynamicSpring; 3],
    icon_toggles: [bool; 4],
    icon_toggle_anim: [DynamicSpring; 4],
    ext_fab_expanded: bool,
    ext_fab_anim: DynamicSpring,
    fab_open: bool,
    fab_anim: DynamicSpring,
    split_open: [bool; 2],
    split_anim: [DynamicSpring; 2],
    group_sel: usize,
    std_group_sel: [bool; 4],
    tabs: usize,
    sec_tabs: usize,
    nav: usize,
    rail: usize,
    drawer: usize,
    segmented: usize,
    menu_sel: usize,
    carousel: usize,
    carousel_mode: usize,
    carousel_pos: DynamicSpring,
    date_day: usize,
    time_am: bool,
    zones: Vec<(Action, Rect, [f32; 4])>,
}

impl Gallery {
    pub fn new() -> Self {
        let scheme = MotionScheme::expressive();
        let fs = scheme.spec(Tempo::Fast, Track::Spatial);
        let ds = scheme.spec(Tempo::Default, Track::Spatial);
        let dfx = scheme.spec(Tempo::Default, Track::Effects);
        let switches = [true, false, true];
        let check_state = [1u8, 0, 2];
        let radio = 0usize;
        let chips = [false, true, true, false];
        let toggle_btns = [true, false, false];
        let icon_toggles = [true, false, true, false];
        Self {
            pal: Palette::dark(),
            u: 1.0,
            scroll: 0.0,
            content_h: 1000.0,
            view_h: 1000.0,
            down: None,
            down_moved: false,
            drag_last_y: 0.0,
            slider_drag: false,
            pressed: None,
            last_release: None,
            ripples: Vec::new(),
            start: Instant::now(),
            t: 0.0,
            morph: DynamicSpring::new(0.0, 0.0, fs),
            morph_key: 0,
            group_press: DynamicSpring::new(0.0, 0.0, fs),
            group_pressed_idx: None,
            slider_thumb: DynamicSpring::new(0.0, 0.0, fs),
            slider: 0.62,
            focus: DynamicSpring::new(0.0, 1.0, dfx),
            switches,
            switch_anim: std::array::from_fn(|i| {
                let v = f32::from(switches[i]);
                DynamicSpring::new(v, v, fs)
            }),
            check_state,
            check_draw: std::array::from_fn(|i| {
                let v = f32::from(check_state[i] == 1);
                DynamicSpring::new(v, v, ds)
            }),
            check_color: std::array::from_fn(|i| {
                let v = f32::from(check_state[i] != 0);
                DynamicSpring::new(v, v, dfx)
            }),
            check_uncheck_at: [f32::INFINITY; 3],
            radio,
            radio_anim: std::array::from_fn(|i| {
                let v = f32::from(i == radio);
                DynamicSpring::new(v, v, fs)
            }),
            chips,
            chip_anim: std::array::from_fn(|i| {
                let v = f32::from(chips[i]);
                DynamicSpring::new(v, v, fs)
            }),
            toggle_btns,
            toggle_btn_anim: std::array::from_fn(|i| {
                let v = f32::from(toggle_btns[i]);
                DynamicSpring::new(v, v, fs)
            }),
            icon_toggles,
            icon_toggle_anim: std::array::from_fn(|i| {
                let v = f32::from(icon_toggles[i]);
                DynamicSpring::new(v, v, fs)
            }),
            ext_fab_expanded: true,
            ext_fab_anim: DynamicSpring::new(1.0, 1.0, ds),
            fab_open: true,
            fab_anim: DynamicSpring::new(1.0, 1.0, ds),
            split_open: [false, false],
            split_anim: [
                DynamicSpring::new(0.0, 0.0, fs),
                DynamicSpring::new(0.0, 0.0, fs),
            ],
            group_sel: 1,
            std_group_sel: [true, false, true, false],
            tabs: 0,
            sec_tabs: 1,
            nav: 0,
            rail: 0,
            drawer: 0,
            segmented: 0,
            menu_sel: 1,
            carousel: 0,
            carousel_mode: 0,
            carousel_pos: DynamicSpring::new(0.0, 0.0, ds),
            date_day: 15,
            time_am: true,
            zones: Vec::new(),
        }
    }

    pub fn bg(&self) -> [f32; 4] {
        self.pal.bg
    }

    fn unit(width: f32) -> f32 {
        (width / 480.0).max(1.0)
    }

    pub fn step(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 1.0 / 15.0);
        self.morph.step(dt);
        self.group_press.step(dt);
        self.slider_thumb.step(dt);
        self.focus.step(dt);
        self.ext_fab_anim.step(dt);
        self.fab_anim.step(dt);
        self.carousel_pos.step(dt);
        for s in &mut self.switch_anim {
            s.step(dt);
        }
        for s in &mut self.check_draw {
            s.step(dt);
        }
        for s in &mut self.check_color {
            s.step(dt);
        }
        for s in &mut self.radio_anim {
            s.step(dt);
        }
        for s in &mut self.chip_anim {
            s.step(dt);
        }
        for s in &mut self.toggle_btn_anim {
            s.step(dt);
        }
        for s in &mut self.icon_toggle_anim {
            s.step(dt);
        }
        for s in &mut self.split_anim {
            s.step(dt);
        }
        self.t = self.start.elapsed().as_secs_f32();
        for (i, at) in self.check_uncheck_at.iter_mut().enumerate() {
            if self.t >= *at {
                self.check_draw[i].retarget(0.0);
                *at = f32::INFINITY;
            }
        }
        if self.focus.settled() {
            self.focus.retarget(1.0 - self.focus.target);
        }
    }

    pub fn scroll_by(&mut self, dy: f32) {
        let max = (self.content_h - self.view_h).max(0.0);
        self.scroll = (self.scroll + dy).clamp(0.0, max);
    }

    pub fn down(&mut self, x: f32, y: f32) {
        self.down = Some((x, y));
        self.down_moved = false;
        self.drag_last_y = y;
        let hit = self
            .zones
            .iter()
            .rev()
            .find(|(_, r, _)| r.contains(x, y))
            .map(|(a, r, c)| (*a, *r, *c));
        let Some((action, rect, ink)) = hit else {
            return;
        };
        if action == Action::Slider {
            self.slider_drag = true;
            self.slider_thumb.x = 1.0;
            self.slider_thumb.retarget(1.0);
            self.drag_slider_to(x);
            return;
        }
        let bounded = !matches!(
            action,
            Action::Switch(_) | Action::Check(_) | Action::Radio(_)
        );
        self.spawn_ripple(rect, x, y, bounded, ink);
        self.pressed = Some(Pressed {
            action,
            start: self.t,
        });
        if let Action::Button(id) = action {
            self.morph_key = id;
            self.morph.retarget(1.0);
        }
        if let Action::Group(idx) = action {
            self.group_pressed_idx = Some(idx);
            self.group_press.retarget(1.0);
        }
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        if let Some((dx, _)) = self.down {
            if (x - dx).abs() > 8.0 {
                self.cancel_press();
                self.down_moved = true;
            }
            if self.slider_drag {
                self.drag_slider_to(x);
                return;
            }
            let dy = self.drag_last_y - y;
            if dy.abs() > 1.0 {
                self.cancel_press();
                self.down_moved = true;
            }
            self.scroll_by(dy);
            self.drag_last_y = y;
        }
    }

    pub fn up(&mut self, x: f32, y: f32) {
        if self.slider_drag {
            self.slider_drag = false;
            self.slider_thumb.retarget(0.0);
        }
        let tapped = !self.down_moved;
        self.cancel_press();
        self.down = None;
        self.down_moved = false;
        if !tapped {
            return;
        }
        let hit = self
            .zones
            .iter()
            .rev()
            .find(|(_, r, _)| r.contains(x, y))
            .map(|(a, _, _)| *a);
        if let Some(a) = hit {
            self.activate(a);
        }
    }

    fn cancel_press(&mut self) {
        if let Some(p) = self.pressed.take() {
            self.last_release = Some((p.action, self.t));
            for r in &mut self.ripples {
                if r.released.is_none() {
                    r.released = Some(self.t);
                }
            }
        }
        if self.morph.target != 0.0 {
            self.morph.retarget(0.0);
        }
        if self.group_press.target != 0.0 {
            self.group_press.retarget(0.0);
        }
        if self.slider_thumb.target != 0.0 && !self.slider_drag {
            self.slider_thumb.retarget(0.0);
        }
    }

    fn drag_slider_to(&mut self, x: f32) {
        for (a, r, _) in self.zones.iter().rev() {
            if *a == Action::Slider && x >= r.x - 60.0 && x <= r.x + r.w + 60.0 {
                self.slider = ((x - r.x) / r.w).clamp(0.0, 1.0);
                break;
            }
        }
    }

    fn activate(&mut self, action: Action) {
        let scheme = MotionScheme::expressive();
        match action {
            Action::Switch(i) => {
                self.switches[i] = !self.switches[i];
                self.switch_anim[i].retarget(f32::from(self.switches[i]));
            }
            Action::Check(i) => {
                let prev = self.check_state[i];
                let next = (prev + 1) % 3;
                self.check_state[i] = next;
                if next == 0 {
                    self.check_color[i].spec = scheme.spec(Tempo::Fast, Track::Effects);
                    self.check_color[i].retarget(0.0);
                    if prev == 1 {
                        self.check_uncheck_at[i] = self.t + 0.1;
                    }
                } else {
                    self.check_color[i].spec = scheme.spec(Tempo::Default, Track::Effects);
                    self.check_color[i].retarget(1.0);
                    if next == 1 {
                        self.check_draw[i].retarget(1.0);
                        self.check_uncheck_at[i] = f32::INFINITY;
                    }
                }
            }
            Action::Radio(i) => {
                self.radio = i;
                for (j, s) in self.radio_anim.iter_mut().enumerate() {
                    s.retarget(f32::from(j == i));
                }
            }
            Action::Chip(i) => {
                self.chips[i] = !self.chips[i];
                self.chip_anim[i].retarget(f32::from(self.chips[i]));
            }
            Action::ToggleBtn(i) => {
                self.toggle_btns[i] = !self.toggle_btns[i];
                self.toggle_btn_anim[i].retarget(f32::from(self.toggle_btns[i]));
            }
            Action::IconToggle(i) => {
                self.icon_toggles[i] = !self.icon_toggles[i];
                self.icon_toggle_anim[i].retarget(f32::from(self.icon_toggles[i]));
            }
            Action::ExtendedFabToggle => {
                self.ext_fab_expanded = !self.ext_fab_expanded;
                self.ext_fab_anim.retarget(f32::from(self.ext_fab_expanded));
            }
            Action::SplitTrailing(i) => {
                self.split_open[i] = !self.split_open[i];
                self.split_anim[i].retarget(f32::from(self.split_open[i]));
            }
            Action::Tab(i) => self.tabs = i,
            Action::SecondaryTab(i) => self.sec_tabs = i,
            Action::Nav(i) => self.nav = i,
            Action::Rail(i) => self.rail = i,
            Action::Drawer(i) => self.drawer = i,
            Action::Group(i) => self.group_sel = i,
            Action::StdGroup(i) => self.std_group_sel[i] = !self.std_group_sel[i],
            Action::Segmented(i) => self.segmented = i,
            Action::MenuItem(i) => self.menu_sel = i,
            Action::CarouselPage(i) => {
                self.carousel = i;
                self.carousel_pos.retarget(i as f32);
            }
            Action::CarouselMode(m) => {
                self.carousel_mode = m;
                if m == 1 && self.carousel == 0 {
                    self.carousel = 1;
                    self.carousel_pos.retarget(1.0);
                }
            }
            Action::DateDay(d) => self.date_day = d,
            Action::TimeAmPm(am) => self.time_am = am,
            Action::FabMenu => {
                self.fab_open = !self.fab_open;
                self.fab_anim.retarget(f32::from(self.fab_open));
            }
            Action::Button(_) | Action::Slider => {}
        }
    }

    fn zone(&mut self, action: Action, rect: Rect, ink: [f32; 4]) {
        self.zones.push((action, rect, ink));
    }

    fn is_pressed(&self, action: Action) -> bool {
        self.pressed.is_some_and(|p| p.action == action)
    }

    fn spawn_ripple(&mut self, rect: Rect, x: f32, y: f32, bounded: bool, ink: [f32; 4]) {
        self.ripples.push(Ripple {
            rect,
            x,
            y,
            start: self.t,
            released: None,
            bounded,
            ink,
        });
        if self.ripples.len() > 8 {
            self.ripples.remove(0);
        }
    }

    fn draw_ripples(&self, ui: &mut UiRenderer, rect: Rect) {
        let u = self.u;
        let now = self.t;
        for r in &self.ripples {
            if r.rect != rect {
                continue;
            }
            let grow = cubic_bezier_y(
                ((now - r.start) / 0.225).clamp(0.0, 1.0),
                0.2,
                0.0,
                0.0,
                1.0,
            );
            let end_r = if r.bounded {
                (rect.w * rect.w + rect.h * rect.h).sqrt() * 0.5 + 10.0 * u
            } else {
                20.0 * u
            };
            let start_r = if r.bounded {
                0.25 * rect.w.max(rect.h)
            } else {
                0.0
            };
            let radius = lerp(start_r, end_r, grow);
            let mut a = ((now - r.start) / 0.075).clamp(0.0, 1.0);
            if let Some(t0) = r.released {
                a *= 1.0 - ((now - t0) / 0.15).clamp(0.0, 1.0);
            }
            let col = with_alpha(r.ink, 0.10 * a);
            ui.circle(r.x, r.y, radius, col);
            if r.bounded {
                ui.clip_last([rect.x, rect.y, rect.w, rect.h]);
            }
        }
    }

    fn layer(&self, ui: &mut UiRenderer, action: Action, rect: Rect, radius: f32, ink: [f32; 4]) {
        if self.is_pressed(action) {
            ui.rect(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                with_alpha(ink, 0.10),
                r4(radius),
            );
        }
    }

    fn layer_radii(
        &self,
        ui: &mut UiRenderer,
        action: Action,
        rect: Rect,
        radii: [f32; 4],
        ink: [f32; 4],
    ) {
        if self.is_pressed(action) {
            ui.rect(rect.x, rect.y, rect.w, rect.h, with_alpha(ink, 0.10), radii);
        }
    }

    fn layer_circle(
        &self,
        ui: &mut UiRenderer,
        action: Action,
        cx: f32,
        cy: f32,
        r: f32,
        ink: [f32; 4],
    ) {
        if self.is_pressed(action) {
            ui.circle(cx, cy, r, with_alpha(ink, 0.10));
        }
    }

    fn elev_dp(&self, action: Action) -> f32 {
        if let Some(p) = self.pressed.filter(|p| p.action == action) {
            let f = cubic_bezier_y(
                ((self.t - p.start) / 0.12).clamp(0.0, 1.0),
                0.2,
                0.0,
                0.0,
                1.0,
            );
            return 2.0 + 4.0 * f;
        }
        if let Some((_, t0)) = self.last_release.filter(|(a, _)| *a == action) {
            if self.t - t0 < 0.15 {
                let f = cubic_bezier_y((self.t - t0) / 0.15, 0.4, 0.0, 0.6, 1.0);
                return 6.0 - 4.0 * f;
            }
        }
        2.0
    }

    /// Renders a Material 3 Expressive circular progress indicator (flat or wavy) with track gap.
    #[allow(clippy::too_many_arguments)]
    fn draw_circular_progress(
        &self,
        ui: &mut UiRenderer,
        cx: f32,
        cy: f32,
        outer_r: f32,
        stroke_w: f32,
        start_deg: f32,
        sweep_deg: f32,
        wavy: bool,
    ) {
        let u = self.u;
        let sweep_clamped = sweep_deg.clamp(0.0, 360.0);
        let start_rad = start_deg.to_radians();
        let sweep_rad = sweep_clamped.to_radians();
        let mid_r = (outer_r - stroke_w * 0.5).max(1.0);
        // Gap between active indicator caps and track caps: 4dp + stroke cap radius
        let gap_rad = if sweep_clamped > 2.0 && sweep_clamped < 356.0 {
            ((4.0 * u + stroke_w) / mid_r).min(0.42)
        } else {
            0.0
        };
        let track_sweep = (TAU - sweep_rad - 2.0 * gap_rad).max(0.0);
        if track_sweep > 0.02 {
            ui.arc_ring(
                cx,
                cy,
                outer_r,
                stroke_w,
                start_rad + sweep_rad + gap_rad,
                track_sweep,
                self.pal.secondary_container,
            );
        }
        if sweep_rad > 0.01 {
            if wavy {
                let amp_factor =
                    ((sweep_clamped / 30.0).min((360.0 - sweep_clamped) / 25.0)).clamp(0.25, 1.0);
                let amp = 2.5 * u * amp_factor;
                let phase = -self.t * 6.5;
                ui.wavy_arc_ring(
                    cx,
                    cy,
                    outer_r,
                    stroke_w,
                    start_rad,
                    sweep_rad,
                    amp,
                    8.0,
                    phase,
                    self.pal.primary,
                );
            } else {
                ui.arc_ring(
                    cx,
                    cy,
                    outer_r,
                    stroke_w,
                    start_rad,
                    sweep_rad,
                    self.pal.primary,
                );
            }
        }
    }

    pub fn draw(&mut self, ui: &mut UiRenderer, width: f32, height: f32) {
        self.view_h = height;
        self.zones.clear();
        let u = Self::unit(width);
        self.u = u;
        let now = self.t;
        self.ripples
            .retain(|r| r.released.is_none_or(|t0| now - t0 < 0.15));
        let pad = 16.0 * u;
        let card_x = 8.0 * u;
        let card_w = width - 16.0 * u;
        let mut y = 76.0 * u - self.scroll;
        for id in ComponentId::ALL {
            let title = Self::component_title(id);
            ui.text(title, pad, y, ts(M3_LABEL_LARGE, self.pal.primary, u));
            let sub = Self::component_subtitle(id);
            let title_w = ui.text_width(title, 14.0 * u, true, 0.1 / 14.0);
            ui.text(
                sub,
                pad + title_w + 10.0 * u,
                y + 1.0 * u,
                ts(M3_LABEL_MEDIUM, self.pal.sub, u),
            );
            let body_h = Self::body_height(id, u);
            let card_y = y + 24.0 * u;
            ui.rect_styled(
                card_x,
                card_y,
                card_w,
                body_h + 32.0 * u,
                self.pal.card,
                self.pal.outline_variant,
                1.0 * u,
                r4(16.0 * u),
                0.0,
            );
            self.draw_body(
                ui,
                id,
                card_x + 16.0 * u,
                card_y + 16.0 * u,
                card_w - 32.0 * u,
            );
            y = card_y + body_h + 32.0 * u + 20.0 * u;
        }
        self.content_h = y + self.scroll + 16.0 * u;
        self.scroll_by(0.0);
        self.draw_app_bar(ui, width);
    }

    fn component_title(id: ComponentId) -> &'static str {
        match id {
            ComponentId::Button => "Button & ToggleButton",
            ComponentId::IconButton => "IconButton",
            ComponentId::Fab => "FloatingActionButton",
            ComponentId::ExtendedFab => "ExtendedFloatingActionButton",
            ComponentId::FabMenu => "FloatingActionButtonMenu",
            ComponentId::SplitButton => "SplitButtonLayout",
            ComponentId::ButtonGroup => "ButtonGroup (Connected & Standard)",
            ComponentId::Card => "Card (Elevated, Filled, Outlined)",
            ComponentId::Checkbox => "Checkbox",
            ComponentId::Switch => "Switch",
            ComponentId::RadioButton => "RadioButton",
            ComponentId::Slider => "Slider & WavySlider",
            ComponentId::ProgressIndicator => "ProgressIndicator & WavyProgressIndicator",
            ComponentId::LoadingIndicator => "LoadingIndicator & MaterialShapes (35 GPU SDFs)",
            ComponentId::AssistChip => "AssistChip",
            ComponentId::FilterChip => "FilterChip",
            ComponentId::InputChip => "InputChip",
            ComponentId::SuggestionChip => "SuggestionChip",
            ComponentId::Dialog => "BasicAlertDialog",
            ComponentId::BottomSheet => "ModalBottomSheet",
            ComponentId::NavigationBar => "ShortNavigationBar",
            ComponentId::NavigationRail => "WideNavigationRail",
            ComponentId::NavigationDrawer => "ModalNavigationDrawer",
            ComponentId::Scaffold => "Scaffold",
            ComponentId::TopAppBar => "TopAppBar (MediumFlexible)",
            ComponentId::SearchBar => "SearchBar",
            ComponentId::TextField => "TextField & OutlinedTextField",
            ComponentId::Menu => "DropdownMenuPopup (Expressive Grouped)",
            ComponentId::Carousel => "HorizontalMultiBrowse & CenteredHero Carousel",
            ComponentId::DatePicker => "DatePicker",
            ComponentId::TimePicker => "TimePicker",
            ComponentId::Tooltip => "PlainTooltip & RichTooltip",
            ComponentId::Snackbar => "Snackbar",
            ComponentId::Badge => "Badge",
            ComponentId::ListItem => "ListItem (Expressive Segmented List)",
            ComponentId::Tabs => "PrimaryTabRow & SecondaryTabRow",
            ComponentId::SegmentedButton => "SingleChoiceSegmentedButtonRow",
            ComponentId::Divider => "HorizontalDivider",
            ComponentId::PullToRefresh => "PullToRefreshBox",
            ComponentId::SwipeToDismiss => "SwipeToDismissBox",
            ComponentId::Toolbar => "HorizontalFloatingToolbar & FAB",
        }
    }

    fn component_subtitle(id: ComponentId) -> &'static str {
        match id {
            ComponentId::LoadingIndicator => "7-shape spring morph + 35 RoundedPolygons",
            ComponentId::ProgressIndicator => "Linear/Circular + Sinusoidal Wavy Shader",
            ComponentId::Carousel => "28dp corners, dynamic hero squish, uniform height",
            ComponentId::ButtonGroup => "Spring width physics + corner morphing",
            ComponentId::SplitButton => "Asymmetric 20dp/4dp corners + circular morph",
            ComponentId::FabMenu => "Expressive spring menu items + rotating trigger",
            ComponentId::Slider => "16dp track, 4x44dp handle, track gap & wavy track",
            _ => "Material 3 Expressive",
        }
    }

    fn draw_app_bar(&mut self, ui: &mut UiRenderer, width: f32) {
        let u = self.u;
        ui.rect(0.0, 0.0, width, 64.0 * u, self.pal.bg, r4(0.0));
        // Expressive M3 shape badge in top app bar
        let (sa, sb, mt, rot, _) = loading_morph_state(self.t, 0);
        ui.m3_shape_morph(
            28.0 * u,
            32.0 * u,
            30.0 * u,
            sa,
            sb,
            mt,
            rot,
            self.pal.primary_container,
            self.pal.primary,
            1.2 * u,
        );
        ui.text(
            "Material 3 Expressive",
            52.0 * u,
            18.0 * u,
            ts(M3_TITLE_LARGE, self.pal.text, u),
        );
        self.icon_button(
            ui,
            101,
            width - 92.0 * u,
            12.0 * u,
            "search",
            IconBtn::Tonal,
        );
        self.icon_button(
            ui,
            102,
            width - 46.0 * u,
            12.0 * u,
            "more_vert",
            IconBtn::Standard,
        );
        ui.rect(
            0.0,
            63.0 * u,
            width,
            1.0 * u,
            self.pal.outline_variant,
            r4(0.5 * u),
        );
    }

    fn body_height(id: ComponentId, u: f32) -> f32 {
        match id {
            ComponentId::Button => 148.0 * u,
            ComponentId::IconButton => 104.0 * u,
            ComponentId::Fab => 84.0 * u,
            ComponentId::ExtendedFab => 64.0 * u,
            ComponentId::FabMenu => 272.0 * u,
            ComponentId::SplitButton => 52.0 * u,
            ComponentId::ButtonGroup => 100.0 * u,
            ComponentId::Card => 176.0 * u,
            ComponentId::Checkbox => 120.0 * u,
            ComponentId::Switch => 136.0 * u,
            ComponentId::RadioButton => 120.0 * u,
            ComponentId::Slider => 132.0 * u,
            ComponentId::ProgressIndicator => 168.0 * u,
            ComponentId::LoadingIndicator => 136.0 * u,
            ComponentId::AssistChip
            | ComponentId::FilterChip
            | ComponentId::InputChip
            | ComponentId::SuggestionChip => 44.0 * u,
            ComponentId::Dialog => 224.0 * u,
            ComponentId::BottomSheet => 188.0 * u,
            ComponentId::NavigationBar => 80.0 * u,
            ComponentId::NavigationRail => 232.0 * u,
            ComponentId::NavigationDrawer => 288.0 * u,
            ComponentId::Scaffold => 232.0 * u,
            ComponentId::TopAppBar => 104.0 * u,
            ComponentId::SearchBar => 64.0 * u,
            ComponentId::TextField => 148.0 * u,
            ComponentId::Menu => 216.0 * u,
            ComponentId::Carousel => 218.0 * u,
            ComponentId::DatePicker => 296.0 * u,
            ComponentId::TimePicker => 212.0 * u,
            ComponentId::Tooltip => 124.0 * u,
            ComponentId::Snackbar => 64.0 * u,
            ComponentId::Badge => 56.0 * u,
            ComponentId::ListItem => 224.0 * u,
            ComponentId::Tabs => 118.0 * u,
            ComponentId::SegmentedButton => 48.0 * u,
            ComponentId::Divider => 52.0 * u,
            ComponentId::PullToRefresh => 72.0 * u,
            ComponentId::SwipeToDismiss => 64.0 * u,
            ComponentId::Toolbar => 64.0 * u,
        }
    }

    fn draw_button(&mut self, ui: &mut UiRenderer, id: u32, x: f32, y: f32, spec: BtnSpec) -> f32 {
        let u = self.u;
        let label = spec.label;
        let icon = spec.icon;
        let variant = spec.variant;
        let h = 40.0 * u;
        let ink = match variant {
            Btn::Filled => self.pal.on_primary,
            Btn::Tonal => self.pal.on_secondary_container,
            Btn::Outlined | Btn::Elevated | Btn::Text => self.pal.primary,
        };
        let fill = match variant {
            Btn::Filled => Some(self.pal.primary),
            Btn::Tonal => Some(self.pal.secondary_container),
            Btn::Elevated => Some(self.pal.card_hi),
            Btn::Outlined | Btn::Text => None,
        };
        let style = ts(M3_LABEL_LARGE, ink, u);
        let tw = ui.text_width(label, style.px, style.medium, style.tracking_em);
        let pad_h = if variant == Btn::Text {
            12.0 * u
        } else {
            20.0 * u
        };
        let icon_w = if icon.is_some() { 26.0 * u } else { 0.0 };
        let mut w = pad_h + icon_w + tw + pad_h;
        if !matches!(variant, Btn::Text) && w < 58.0 * u {
            w = 58.0 * u;
        }
        // M3 Expressive button shapes morph from Full pill (20dp) to Small corner (8dp) when pressed
        let radius = if self.morph_key == id {
            lerp(h * 0.5, 8.0 * u, self.morph.x.clamp(0.0, 1.0))
        } else {
            h * 0.5
        };
        if variant == Btn::Elevated {
            ui.rect(
                x,
                y + 2.0 * u,
                w,
                h,
                with_alpha(self.pal.bg, 0.55),
                r4(radius),
            );
        }
        if let Some(f) = fill {
            ui.rect(x, y, w, h, f, r4(radius));
        }
        if variant == Btn::Outlined {
            ui.stroke(x, y, w, h, self.pal.outline, r4(radius), 1.0 * u);
        }
        self.layer(ui, Action::Button(id), Rect { x, y, w, h }, radius, ink);
        self.draw_ripples(ui, Rect { x, y, w, h });
        if let Some(name) = icon {
            icon_c(ui, name, x + pad_h + 9.0 * u, y + h * 0.5, 18.0 * u, ink);
        }
        ui.text(
            label,
            x + pad_h + icon_w,
            y + h * 0.5 - style.line_h * 0.5,
            style,
        );
        self.zone(Action::Button(id), Rect { x, y, w, h }, ink);
        w
    }

    fn draw_toggle_button(
        &mut self,
        ui: &mut UiRenderer,
        idx: usize,
        x: f32,
        y: f32,
        label: &str,
        icon: &str,
    ) -> f32 {
        let u = self.u;
        let h = 40.0 * u;
        let anim = self.toggle_btn_anim[idx].x.clamp(0.0, 1.0);
        let checked = self.toggle_btns[idx];
        let fill = mix(self.pal.card_hi, self.pal.primary, anim);
        let ink = mix(self.pal.text, self.pal.on_primary, anim);
        let style = ts(M3_LABEL_LARGE, ink, u);
        let tw = ui.text_width(label, style.px, style.medium, style.tracking_em);
        let pad_h = 16.0 * u;
        let icon_w = 24.0 * u;
        let w = pad_h + icon_w + tw + pad_h;
        // M3 Expressive ToggleButton: unchecked = full pill (20dp), checked = 12dp rounded rect, pressed = 8dp
        let pressed_f = if self.is_pressed(Action::ToggleBtn(idx)) {
            1.0
        } else {
            0.0
        };
        let base_r = lerp(20.0 * u, 12.0 * u, anim);
        let radius = lerp(base_r, 8.0 * u, pressed_f);
        let rect = Rect { x, y, w, h };
        ui.rect(x, y, w, h, fill, r4(radius));
        if !checked {
            ui.stroke(
                x,
                y,
                w,
                h,
                with_alpha(self.pal.outline_variant, 1.0 - anim),
                r4(radius),
                1.0 * u,
            );
        }
        self.layer(ui, Action::ToggleBtn(idx), rect, radius, ink);
        self.draw_ripples(ui, rect);
        icon_c(
            ui,
            if checked { "check" } else { icon },
            x + pad_h + 8.0 * u,
            y + h * 0.5,
            18.0 * u,
            ink,
        );
        ui.text(
            label,
            x + pad_h + icon_w,
            y + h * 0.5 - style.line_h * 0.5,
            style,
        );
        self.zone(Action::ToggleBtn(idx), rect, ink);
        w
    }

    fn icon_button(
        &mut self,
        ui: &mut UiRenderer,
        id: u32,
        x: f32,
        y: f32,
        icon: &str,
        variant: IconBtn,
    ) {
        self.icon_button_sized(ui, id, x, y, 40.0 * self.u, 40.0 * self.u, icon, variant);
    }

    #[allow(clippy::too_many_arguments)]
    fn icon_button_sized(
        &mut self,
        ui: &mut UiRenderer,
        id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        icon: &str,
        variant: IconBtn,
    ) {
        let u = self.u;
        let (fill, border, ink) = match variant {
            IconBtn::Standard => (None, false, self.pal.text),
            IconBtn::Filled => (Some(self.pal.primary), false, self.pal.on_primary),
            IconBtn::Tonal => (
                Some(self.pal.secondary_container),
                false,
                self.pal.on_secondary_container,
            ),
            IconBtn::Outlined => (None, true, self.pal.primary),
        };
        let base_r = w.min(h) * 0.5;
        let radius = if self.morph_key == id {
            lerp(base_r, 8.0 * u, self.morph.x.clamp(0.0, 1.0))
        } else {
            base_r
        };
        if let Some(f) = fill {
            ui.rect(x, y, w, h, f, r4(radius));
        }
        if border {
            ui.stroke(x, y, w, h, self.pal.outline, r4(radius), 1.0 * u);
        }
        let rect = Rect { x, y, w, h };
        self.layer(ui, Action::Button(id), rect, radius, ink);
        self.draw_ripples(ui, rect);
        icon_c(ui, icon, x + w * 0.5, y + h * 0.5, 22.0 * u, ink);
        self.zone(Action::Button(id), rect, ink);
    }

    #[allow(clippy::too_many_arguments)]
    fn fab_styled(
        &mut self,
        ui: &mut UiRenderer,
        action: Action,
        x: f32,
        y: f32,
        size: f32,
        radius: f32,
        fill: [f32; 4],
        ink: [f32; 4],
        icon: &str,
        icon_px: f32,
    ) -> f32 {
        let u = self.u;
        let e = self.elev_dp(action) * u;
        let rect = Rect {
            x,
            y,
            w: size,
            h: size,
        };
        ui.rect(
            x,
            y + e,
            size,
            size,
            with_alpha(self.pal.bg, 0.65),
            r4(radius),
        );
        ui.rect(x, y, size, size, fill, r4(radius));
        self.layer(ui, action, rect, radius, ink);
        self.draw_ripples(ui, rect);
        icon_c(ui, icon, x + size * 0.5, y + size * 0.5, icon_px, ink);
        self.zone(action, rect, ink);
        size
    }

    fn draw_checkbox(&mut self, ui: &mut UiRenderer, i: usize, x: f32, ry: f32, label: &str) {
        let u = self.u;
        let st = self.check_state[i];
        let ca = if st == 0 {
            0.0
        } else {
            self.check_color[i].x.clamp(0.0, 1.0)
        };
        let cf = self.check_draw[i].x.clamp(0.0, 1.0);
        let bx = x + 11.0 * u;
        let by = ry + 11.0 * u;
        let hit = Rect {
            x,
            y: ry,
            w: 40.0 * u,
            h: 40.0 * u,
        };
        self.layer_circle(
            ui,
            Action::Check(i),
            x + 20.0 * u,
            ry + 20.0 * u,
            20.0 * u,
            self.pal.text,
        );
        self.draw_ripples(ui, hit);
        if ca > 0.0 {
            ui.rect(
                bx,
                by,
                18.0 * u,
                18.0 * u,
                with_alpha(self.pal.primary, ca),
                r4(3.0 * u),
            );
        }
        if ca < 1.0 {
            ui.stroke(
                bx,
                by,
                18.0 * u,
                18.0 * u,
                with_alpha(self.pal.outline, 1.0 - ca),
                r4(3.0 * u),
                2.0 * u,
            );
        }
        if st == 2 {
            ui.rect(
                bx + 4.0 * u,
                by + 8.0 * u,
                10.0 * u,
                2.0 * u,
                with_alpha(self.pal.on_primary, ca),
                r4(1.0 * u),
            );
        } else if cf > 0.02 {
            // Analytical 2-segment vector checkmark drawn with `stroke_line` (no broken icon clipping)
            let p0 = [bx + 4.0 * u, by + 9.2 * u];
            let p1 = [bx + 7.6 * u, by + 12.8 * u];
            let p2 = [bx + 14.2 * u, by + 5.6 * u];
            let seg1 = (cf / 0.42).clamp(0.0, 1.0);
            let e1 = [lerp(p0[0], p1[0], seg1), lerp(p0[1], p1[1], seg1)];
            ui.stroke_line(p0, e1, 2.0 * u, self.pal.on_primary);
            if cf > 0.42 {
                let seg2 = ((cf - 0.42) / 0.58).clamp(0.0, 1.0);
                let e2 = [lerp(p1[0], p2[0], seg2), lerp(p1[1], p2[1], seg2)];
                ui.stroke_line(p1, e2, 2.0 * u, self.pal.on_primary);
            }
        }
        ui.text(
            label,
            x + 48.0 * u,
            ry + 8.0 * u,
            ts(M3_BODY_LARGE, self.pal.text, u),
        );
        self.zone(
            Action::Check(i),
            Rect {
                x,
                y: ry,
                w: 240.0 * u,
                h: 40.0 * u,
            },
            self.pal.text,
        );
    }

    fn draw_switch(
        &mut self,
        ui: &mut UiRenderer,
        i: usize,
        x: f32,
        ry: f32,
        label: &str,
        with_icons: bool,
    ) {
        let u = self.u;
        let on = self.switches[i];
        let anim = self.switch_anim[i].x.clamp(0.0, 1.0);
        let pressed = self.is_pressed(Action::Switch(i));
        let ty = ry + 6.0 * u;
        let cy = ry + 22.0 * u;
        let track_col = mix(self.pal.card_highest, self.pal.primary, anim);
        ui.rect(x, ty, 52.0 * u, 32.0 * u, track_col, r4(16.0 * u));
        if anim < 0.99 {
            ui.stroke(
                x,
                ty,
                52.0 * u,
                32.0 * u,
                with_alpha(self.pal.outline, 1.0 - anim),
                r4(16.0 * u),
                2.0 * u,
            );
        }
        let off_size = if with_icons { 24.0 * u } else { 16.0 * u };
        let size = if pressed {
            28.0 * u
        } else {
            lerp(off_size, 24.0 * u, anim)
        };
        let cx = lerp(16.0 * u, 36.0 * u, anim);
        self.layer_circle(ui, Action::Switch(i), x + cx, cy, 20.0 * u, self.pal.text);
        self.draw_ripples(
            ui,
            Rect {
                x,
                y: ry,
                w: 52.0 * u,
                h: 44.0 * u,
            },
        );
        let thumb = mix(self.pal.outline, self.pal.on_primary, anim);
        ui.circle(x + cx, cy, size * 0.5, thumb);
        if on && (with_icons || anim > 0.5) {
            icon_c(
                ui,
                "check",
                x + cx,
                cy,
                16.0 * u,
                self.pal.primary_container,
            );
        } else if !on && with_icons {
            icon_c(ui, "close", x + cx, cy, 16.0 * u, self.pal.card_highest);
        }
        ui.text(
            label,
            x + 68.0 * u,
            ry + 10.0 * u,
            ts(M3_BODY_LARGE, self.pal.text, u),
        );
        self.zone(
            Action::Switch(i),
            Rect {
                x,
                y: ry,
                w: 240.0 * u,
                h: 44.0 * u,
            },
            self.pal.text,
        );
    }

    fn draw_radio(&mut self, ui: &mut UiRenderer, i: usize, x: f32, ry: f32, label: &str) {
        let u = self.u;
        let sel = self.radio == i;
        let cx = x + 20.0 * u;
        let cy = ry + 20.0 * u;
        let ring = if sel {
            self.pal.primary
        } else {
            self.pal.outline
        };
        self.layer_circle(ui, Action::Radio(i), cx, cy, 20.0 * u, self.pal.text);
        self.draw_ripples(
            ui,
            Rect {
                x,
                y: ry,
                w: 40.0 * u,
                h: 40.0 * u,
            },
        );
        ui.stroke(
            cx - 10.0 * u,
            cy - 10.0 * u,
            20.0 * u,
            20.0 * u,
            ring,
            r4(10.0 * u),
            2.0 * u,
        );
        let dot = self.radio_anim[i].x.clamp(0.0, 1.2);
        if dot > 0.02 {
            ui.circle(cx, cy, 5.0 * u * dot, self.pal.primary);
        }
        ui.text(
            label,
            x + 48.0 * u,
            ry + 8.0 * u,
            ts(M3_BODY_LARGE, self.pal.text, u),
        );
        self.zone(
            Action::Radio(i),
            Rect {
                x,
                y: ry,
                w: 240.0 * u,
                h: 40.0 * u,
            },
            self.pal.text,
        );
    }

    fn draw_chip(
        &mut self,
        ui: &mut UiRenderer,
        i: usize,
        x: f32,
        y: f32,
        label: &str,
        icons: (Option<&str>, Option<&str>),
    ) -> f32 {
        let (leading, trailing) = icons;
        let u = self.u;
        let sel = self.chips[i];
        let anim = self.chip_anim[i].x.clamp(0.0, 1.0);
        let radius = lerp(8.0 * u, 16.0 * u, anim);
        let ink = if sel {
            self.pal.on_secondary_container
        } else {
            self.pal.text
        };
        let style = ts(M3_LABEL_LARGE, ink, u);
        let tw = ui.text_width(label, style.px, style.medium, style.tracking_em);
        let lead_w = if leading.is_some() { 24.0 * u } else { 0.0 };
        let trail_w = if trailing.is_some() { 24.0 * u } else { 0.0 };
        let w = 12.0 * u + lead_w + tw + trail_w + 12.0 * u;
        let h = 32.0 * u;
        let rect = Rect { x, y, w, h };
        if sel {
            ui.rect(x, y, w, h, self.pal.secondary_container, r4(radius));
        } else {
            ui.stroke(x, y, w, h, self.pal.outline, r4(radius), 1.0 * u);
        }
        self.layer(ui, Action::Chip(i), rect, radius, self.pal.text);
        self.draw_ripples(ui, rect);
        if let Some(ic) = leading {
            icon_c(
                ui,
                ic,
                x + 12.0 * u + 8.0 * u,
                y + h * 0.5,
                18.0 * u,
                if sel {
                    self.pal.on_secondary_container
                } else {
                    self.pal.primary
                },
            );
        }
        let tx = x + 12.0 * u + lead_w;
        ui.text(label, tx, y + h * 0.5 - style.line_h * 0.5, style);
        if let Some(ic) = trailing {
            icon_c(
                ui,
                ic,
                tx + tw + 6.0 * u + 8.0 * u,
                y + h * 0.5,
                18.0 * u,
                self.pal.sub,
            );
        }
        self.zone(Action::Chip(i), rect, self.pal.text);
        w
    }

    fn draw_body(&mut self, ui: &mut UiRenderer, id: ComponentId, x: f32, y: f32, w: f32) {
        let u = self.u;
        match id {
            ComponentId::Button => {
                // Row 1: Standard 5 M3 variants with press corner morph
                let mut bx = x;
                bx += self.draw_button(
                    ui,
                    1,
                    bx,
                    y,
                    BtnSpec {
                        label: "Filled",
                        icon: None,
                        variant: Btn::Filled,
                    },
                ) + 8.0 * u;
                bx += self.draw_button(
                    ui,
                    2,
                    bx,
                    y,
                    BtnSpec {
                        label: "Tonal",
                        icon: None,
                        variant: Btn::Tonal,
                    },
                ) + 8.0 * u;
                bx += self.draw_button(
                    ui,
                    3,
                    bx,
                    y,
                    BtnSpec {
                        label: "Outlined",
                        icon: None,
                        variant: Btn::Outlined,
                    },
                ) + 8.0 * u;
                bx += self.draw_button(
                    ui,
                    4,
                    bx,
                    y,
                    BtnSpec {
                        label: "Elevated",
                        icon: None,
                        variant: Btn::Elevated,
                    },
                ) + 4.0 * u;
                self.draw_button(
                    ui,
                    5,
                    bx,
                    y,
                    BtnSpec {
                        label: "Text",
                        icon: None,
                        variant: Btn::Text,
                    },
                );

                // Row 2: M3 Expressive ToggleButtons (pill <-> rounded square morph on toggle)
                let mut tx = x;
                let ty = y + 48.0 * u;
                let t_specs = [
                    ("Saved", "bookmark"),
                    ("Starred", "star"),
                    ("Liked", "favorite"),
                ];
                for (i, (lb, ic)) in t_specs.into_iter().enumerate() {
                    tx += self.draw_toggle_button(ui, i, tx, ty, lb, ic) + 8.0 * u;
                }

                // Row 3: M3 Expressive Size Tiers (XS 32dp, S 40dp, M 52dp)
                let sy = y + 96.0 * u;
                // XS (32dp)
                ui.rect(
                    x,
                    sy + 10.0 * u,
                    74.0 * u,
                    32.0 * u,
                    self.pal.secondary_container,
                    r4(16.0 * u),
                );
                text_c(
                    ui,
                    "XS 32dp",
                    x + 37.0 * u,
                    sy + 18.0 * u,
                    ts(M3_LABEL_MEDIUM, self.pal.on_secondary_container, u),
                );
                // S (40dp)
                ui.rect(
                    x + 84.0 * u,
                    sy + 6.0 * u,
                    84.0 * u,
                    40.0 * u,
                    self.pal.primary_container,
                    r4(20.0 * u),
                );
                text_c(
                    ui,
                    "S 40dp",
                    x + 126.0 * u,
                    sy + 16.0 * u,
                    ts(M3_LABEL_LARGE, self.pal.on_primary_container, u),
                );
                // M (52dp Expressive hero CTA)
                let mw = (w - 180.0 * u).max(160.0 * u);
                let mx = x + 178.0 * u;
                let m_id = 6;
                let m_r = if self.morph_key == m_id {
                    lerp(26.0 * u, 12.0 * u, self.morph.x.clamp(0.0, 1.0))
                } else {
                    26.0 * u
                };
                ui.rect(mx, sy, mw, 52.0 * u, self.pal.primary, r4(m_r));
                let m_rect = Rect {
                    x: mx,
                    y: sy,
                    w: mw,
                    h: 52.0 * u,
                };
                self.layer(ui, Action::Button(m_id), m_rect, m_r, self.pal.on_primary);
                self.draw_ripples(ui, m_rect);
                icon_c(
                    ui,
                    "play_arrow",
                    mx + 26.0 * u,
                    sy + 26.0 * u,
                    24.0 * u,
                    self.pal.on_primary,
                );
                ui.text(
                    "Expressive M (56dp)",
                    mx + 44.0 * u,
                    sy + 14.0 * u,
                    ts(M3_TITLE_MEDIUM, self.pal.on_primary, u),
                );
                self.zone(Action::Button(m_id), m_rect, self.pal.on_primary);
            }
            ComponentId::IconButton => {
                // Row 1: Standard, Filled, Tonal, Outlined + Wide (52x40) & Narrow (32x40) M3E shapes
                let variants = [
                    IconBtn::Standard,
                    IconBtn::Filled,
                    IconBtn::Tonal,
                    IconBtn::Outlined,
                ];
                let icons = ["favorite", "add", "settings", "share"];
                for (i, v) in variants.into_iter().enumerate() {
                    self.icon_button(
                        ui,
                        10 + i as u32,
                        x + i as f32 * 50.0 * u,
                        y + 4.0 * u,
                        icons[i],
                        v,
                    );
                }
                // Wide Filled IconButton (52x40dp)
                self.icon_button_sized(
                    ui,
                    14,
                    x + 206.0 * u,
                    y + 4.0 * u,
                    56.0 * u,
                    40.0 * u,
                    "mic",
                    IconBtn::Filled,
                );
                // Narrow Tonal IconButton (32x40dp)
                self.icon_button_sized(
                    ui,
                    15,
                    x + 272.0 * u,
                    y + 4.0 * u,
                    32.0 * u,
                    40.0 * u,
                    "more_vert",
                    IconBtn::Tonal,
                );

                // Row 2: Interactive Toggleable IconButtons (morph between circle 20dp and square 12dp)
                let ty = y + 56.0 * u;
                let t_icons = ["favorite", "bookmark", "notifications", "wifi"];
                for (i, ic) in t_icons.into_iter().enumerate() {
                    let ix = x + i as f32 * 52.0 * u;
                    let anim = self.icon_toggle_anim[i].x.clamp(0.0, 1.0);
                    let r = lerp(20.0 * u, 12.0 * u, anim);
                    let fill = mix(self.pal.card_hi, self.pal.primary, anim);
                    let ink = mix(self.pal.text, self.pal.on_primary, anim);
                    let rect = Rect {
                        x: ix,
                        y: ty,
                        w: 40.0 * u,
                        h: 40.0 * u,
                    };
                    ui.rect(ix, ty, 40.0 * u, 40.0 * u, fill, r4(r));
                    self.layer(ui, Action::IconToggle(i), rect, r, ink);
                    self.draw_ripples(ui, rect);
                    icon_c(ui, ic, ix + 20.0 * u, ty + 20.0 * u, 22.0 * u, ink);
                    self.zone(Action::IconToggle(i), rect, ink);
                }
                ui.text(
                    "Tap row 2 to morph toggle shapes",
                    x + 216.0 * u,
                    ty + 12.0 * u,
                    ts(M3_BODY_SMALL, self.pal.sub, u),
                );
            }
            ComponentId::Fab => {
                // Small (40dp, r=12), Medium (56dp, r=16), Large (80dp, r=24, icon=36dp) + M3Shape Sunny FAB
                let mut fx = x;
                fx += self.fab_styled(
                    ui,
                    Action::Button(24),
                    fx,
                    y + 20.0 * u,
                    40.0 * u,
                    12.0 * u,
                    self.pal.secondary_container,
                    self.pal.on_secondary_container,
                    "edit",
                    20.0 * u,
                ) + 16.0 * u;
                fx += self.fab_styled(
                    ui,
                    Action::Button(20),
                    fx,
                    y + 12.0 * u,
                    56.0 * u,
                    16.0 * u,
                    self.pal.primary_container,
                    self.pal.on_primary_container,
                    "add",
                    24.0 * u,
                ) + 16.0 * u;
                fx += self.fab_styled(
                    ui,
                    Action::Button(25),
                    fx,
                    y,
                    80.0 * u,
                    24.0 * u,
                    self.pal.tertiary_container,
                    self.pal.on_tertiary_container,
                    "palette",
                    36.0 * u,
                ) + 20.0 * u;
                // Expressive Cookie9Sided / Sunny morphing FAB
                let cx = fx + 32.0 * u;
                let cy = y + 40.0 * u;
                ui.m3_shape(
                    cx,
                    cy,
                    64.0 * u,
                    MaterialShape::Cookie9Sided,
                    self.t * 0.35,
                    self.pal.primary,
                    [0.0; 4],
                    0.0,
                );
                icon_c(ui, "star", cx, cy, 26.0 * u, self.pal.on_primary);
            }
            ComponentId::ExtendedFab => {
                // Interactive spring-expanding ExtendedFAB + Tonal ExtendedFAB
                let anim = self.ext_fab_anim.x.clamp(0.0, 1.0);
                let ink = self.pal.on_primary_container;
                let style = ts(M3_LABEL_LARGE, ink, u);
                let tw = ui.text_width("New message", style.px, style.medium, style.tracking_em);
                let full_w = 16.0 * u + 24.0 * u + 12.0 * u + tw + 20.0 * u;
                let fw = lerp(56.0 * u, full_w, anim);
                let e = self.elev_dp(Action::ExtendedFabToggle) * u;
                let rect = Rect {
                    x,
                    y: y + 4.0 * u,
                    w: fw,
                    h: 56.0 * u,
                };
                ui.rect(
                    x,
                    rect.y + e,
                    fw,
                    56.0 * u,
                    with_alpha(self.pal.bg, 0.65),
                    r4(16.0 * u),
                );
                ui.rect(
                    x,
                    rect.y,
                    fw,
                    56.0 * u,
                    self.pal.primary_container,
                    r4(16.0 * u),
                );
                self.layer(ui, Action::ExtendedFabToggle, rect, 16.0 * u, ink);
                self.draw_ripples(ui, rect);
                icon_c(ui, "edit", x + 28.0 * u, rect.y + 28.0 * u, 24.0 * u, ink);
                if anim > 0.15 {
                    ui.push_clip([x, rect.y, fw - 8.0 * u, 56.0 * u]);
                    ui.text(
                        "New message",
                        x + 52.0 * u,
                        rect.y + 28.0 * u - style.line_h * 0.5,
                        ts(M3_LABEL_LARGE, with_alpha(ink, anim), u),
                    );
                    ui.pop_clip();
                }
                self.zone(Action::ExtendedFabToggle, rect, ink);

                // Secondary Tonal Extended FAB
                let sx = x + fw + 14.0 * u;
                let s_ink = self.pal.on_secondary_container;
                let s_style = ts(M3_LABEL_LARGE, s_ink, u);
                let stw =
                    ui.text_width("Navigate", s_style.px, s_style.medium, s_style.tracking_em);
                let sw = 16.0 * u + 24.0 * u + 10.0 * u + stw + 20.0 * u;
                if sx + sw <= x + w {
                    let s_rect = Rect {
                        x: sx,
                        y: y + 4.0 * u,
                        w: sw,
                        h: 56.0 * u,
                    };
                    ui.rect(
                        sx,
                        s_rect.y,
                        sw,
                        56.0 * u,
                        self.pal.secondary_container,
                        r4(16.0 * u),
                    );
                    self.layer(ui, Action::Button(22), s_rect, 16.0 * u, s_ink);
                    self.draw_ripples(ui, s_rect);
                    icon_c(
                        ui,
                        "explore",
                        sx + 28.0 * u,
                        s_rect.y + 28.0 * u,
                        24.0 * u,
                        s_ink,
                    );
                    ui.text(
                        "Navigate",
                        sx + 50.0 * u,
                        s_rect.y + 28.0 * u - s_style.line_h * 0.5,
                        s_style,
                    );
                    self.zone(Action::Button(22), s_rect, s_ink);
                }
            }
            ComponentId::FabMenu => {
                // Authentic M3 Expressive FloatingActionButtonMenu:
                // Full-pill items (50dp height, r=25dp) in primary_container containing icon + label,
                // expanding upward with spring animation from a morphing ToggleFloatingActionButton!
                let open_t = self.fab_anim.x.clamp(0.0, 1.0);
                let items = [
                    ("flight", "Book flight"),
                    ("hotel", "Reserve hotel"),
                    ("explore", "Explore map"),
                    ("share", "Share itinerary"),
                ];
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let stagger = ((open_t * 1.35) - (3 - i) as f32 * 0.10).clamp(0.0, 1.0);
                    if stagger <= 0.02 {
                        continue;
                    }
                    let ih = 48.0 * u;
                    let iy = y + i as f32 * 52.0 * u + (1.0 - stagger) * 18.0 * u;
                    let ink = with_alpha(self.pal.on_primary_container, stagger);
                    let fill = with_alpha(self.pal.primary_container, stagger);
                    let style = ts(M3_LABEL_LARGE, ink, u);
                    let tw = ui.text_width(lb, style.px, style.medium, style.tracking_em);
                    let full_iw = 18.0 * u + 22.0 * u + 10.0 * u + tw + 20.0 * u;
                    let iw = lerp(48.0 * u, full_iw, stagger);
                    let r = ih * 0.5;
                    let rect = Rect {
                        x,
                        y: iy,
                        w: iw,
                        h: ih,
                    };
                    ui.rect(x, iy, iw, ih, fill, r4(r));
                    icon_c(ui, ic, x + 26.0 * u, iy + ih * 0.5, 20.0 * u, ink);
                    if stagger > 0.3 {
                        ui.push_clip([x, iy, iw - 10.0 * u, ih]);
                        ui.text(lb, x + 46.0 * u, iy + ih * 0.5 - style.line_h * 0.5, style);
                        ui.pop_clip();
                    }
                    let act = Action::Button(200 + i as u32);
                    self.layer(ui, act, rect, r, self.pal.on_primary_container);
                    self.draw_ripples(ui, rect);
                    self.zone(act, rect, self.pal.on_primary_container);
                }
                let fy = y + 214.0 * u;
                let fab_r = lerp(16.0 * u, 28.0 * u, open_t);
                let fab_fill = mix(self.pal.primary_container, self.pal.primary, open_t);
                let fab_ink = mix(self.pal.on_primary_container, self.pal.on_primary, open_t);
                self.fab_styled(
                    ui,
                    Action::FabMenu,
                    x,
                    fy,
                    56.0 * u,
                    fab_r,
                    fab_fill,
                    fab_ink,
                    if open_t > 0.5 { "close" } else { "add" },
                    24.0 * u,
                );
                ui.text(
                    "Tap trigger FAB to toggle Expressive menu",
                    x + 72.0 * u,
                    fy + 18.0 * u,
                    ts(M3_BODY_SMALL, self.pal.sub, u),
                );
            }
            ComponentId::SplitButton => {
                // Authentic M3 Expressive SplitButtonLayout:
                // Leading button ([20, 4, 4, 20] radii) + 2dp real gap + Trailing button (48x40dp,
                // [4, 20, 20, 4] when closed, morphing to full 20dp circle when open!)
                let h = 40.0 * u;
                let variants = [
                    (
                        0usize,
                        "Send mail",
                        Some("send"),
                        self.pal.primary,
                        self.pal.on_primary,
                        60u32,
                    ),
                    (
                        1usize,
                        "Edit draft",
                        Some("edit"),
                        self.pal.secondary_container,
                        self.pal.on_secondary_container,
                        62u32,
                    ),
                ];
                let mut sx = x;
                for (idx, label, lead_ic, fill, ink, btn_id) in variants {
                    let style = ts(M3_LABEL_LARGE, ink, u);
                    let ltw = ui.text_width(label, style.px, style.medium, style.tracking_em);
                    let lead_w = 16.0 * u + 24.0 * u + ltw + 16.0 * u;
                    let lead_press = if self.morph_key == btn_id {
                        self.morph.x.clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let inner_r = lerp(4.0 * u, 12.0 * u, lead_press);
                    let lead_radii = [20.0 * u, inner_r, inner_r, 20.0 * u];
                    let lead_rect = Rect {
                        x: sx,
                        y: y + 6.0 * u,
                        w: lead_w,
                        h,
                    };
                    ui.rect(lead_rect.x, lead_rect.y, lead_rect.w, h, fill, lead_radii);
                    self.layer_radii(ui, Action::Button(btn_id), lead_rect, lead_radii, ink);
                    self.draw_ripples(ui, lead_rect);
                    if let Some(ic) = lead_ic {
                        icon_c(
                            ui,
                            ic,
                            lead_rect.x + 24.0 * u,
                            lead_rect.y + h * 0.5,
                            18.0 * u,
                            ink,
                        );
                    }
                    ui.text(
                        label,
                        lead_rect.x + 38.0 * u,
                        lead_rect.y + h * 0.5 - style.line_h * 0.5,
                        style,
                    );
                    self.zone(Action::Button(btn_id), lead_rect, ink);

                    // 2dp real gap + Trailing button (44x40dp)
                    let tx = sx + lead_w + 2.0 * u;
                    let tw = 44.0 * u;
                    let open_anim = self.split_anim[idx].x.clamp(0.0, 1.0);
                    let trail_inner = lerp(4.0 * u, 20.0 * u, open_anim);
                    let trail_radii = [trail_inner, 20.0 * u, 20.0 * u, trail_inner];
                    let trail_rect = Rect {
                        x: tx,
                        y: y + 6.0 * u,
                        w: tw,
                        h,
                    };
                    ui.rect(tx, trail_rect.y, tw, h, fill, trail_radii);
                    self.layer_radii(ui, Action::SplitTrailing(idx), trail_rect, trail_radii, ink);
                    self.draw_ripples(ui, trail_rect);
                    // Draw crisp chevron that rotates 180deg when open
                    let cx = tx + tw * 0.5;
                    let cy = trail_rect.y + h * 0.5;
                    let dy = lerp(2.2 * u, -2.2 * u, open_anim);
                    ui.stroke_line([cx - 4.5 * u, cy - dy], [cx, cy + dy], 1.8 * u, ink);
                    ui.stroke_line([cx, cy + dy], [cx + 4.5 * u, cy - dy], 1.8 * u, ink);
                    self.zone(Action::SplitTrailing(idx), trail_rect, ink);

                    sx = tx + tw + 14.0 * u;
                }
            }
            ComponentId::ButtonGroup => {
                // Row 1: M3 Expressive Connected Button Group with spring neighbor width squishing!
                let labels = ["Day", "Week", "Month", "Year"];
                let n = labels.len();
                let gap = 2.0 * u;
                let base_w = (w - gap * (n as f32 - 1.0)) / n as f32;
                let h = 40.0 * u;
                let press_amt = self.group_press.x.clamp(0.0, 1.2);
                let pressed_idx = self.group_pressed_idx;
                let mut widths = [base_w; 4];
                if let Some(pi) = pressed_idx {
                    if press_amt > 0.001 {
                        let delta = base_w * 0.14 * press_amt;
                        widths[pi] += delta;
                        match pi {
                            0 => widths[1] -= delta,
                            3 => widths[2] -= delta,
                            _ => {
                                widths[pi - 1] -= delta * 0.5;
                                widths[pi + 1] -= delta * 0.5;
                            }
                        }
                    }
                }
                let mut gx = x;
                for (i, label) in labels.into_iter().enumerate() {
                    let sw = widths[i];
                    let sel = self.group_sel == i;
                    let is_p = self.is_pressed(Action::Group(i));
                    let fill = if sel {
                        self.pal.primary
                    } else {
                        self.pal.secondary_container
                    };
                    let ink = if sel {
                        self.pal.on_primary
                    } else {
                        self.pal.on_secondary_container
                    };
                    // Connected ButtonGroup corner specification:
                    // Selected item becomes a full pill (20dp). Unselected items have 20dp outer corners and 8dp inner corners (4dp when pressed).
                    let inner = if is_p { 4.0 * u } else { 8.0 * u };
                    let radii = if sel {
                        r4(20.0 * u)
                    } else if i == 0 {
                        [20.0 * u, inner, inner, 20.0 * u]
                    } else if i == n - 1 {
                        [inner, 20.0 * u, 20.0 * u, inner]
                    } else {
                        r4(inner)
                    };
                    let rect = Rect { x: gx, y, w: sw, h };
                    ui.rect(rect.x, rect.y, rect.w, rect.h, fill, radii);
                    self.layer_radii(ui, Action::Group(i), rect, radii, ink);
                    self.draw_ripples(ui, rect);
                    let style = ts(M3_LABEL_LARGE, ink, u);
                    let tw = ui.text_width(label, style.px, style.medium, style.tracking_em);
                    let ic_w = if sel { 20.0 * u } else { 0.0 };
                    let tx = gx + (sw - tw - ic_w) * 0.5;
                    if sel {
                        icon_c(ui, "check", tx + 7.0 * u, y + h * 0.5, 16.0 * u, ink);
                    }
                    ui.text(label, tx + ic_w, y + h * 0.5 - style.line_h * 0.5, style);
                    self.zone(Action::Group(i), rect, ink);
                    gx += sw + gap;
                }

                // Row 2: M3 Expressive Standard Multi-Select Formatting ButtonGroup
                let icons = [
                    ("format_bold", "Bold"),
                    ("format_italic", "Italic"),
                    ("format_underlined", "Underline"),
                    ("link", "Link"),
                ];
                let ry = y + 52.0 * u;
                let mut sx = x;
                for (i, (ic, lb)) in icons.into_iter().enumerate() {
                    let sel = self.std_group_sel[i];
                    let fill = if sel {
                        self.pal.tertiary_container
                    } else {
                        self.pal.card_hi
                    };
                    let ink = if sel {
                        self.pal.on_tertiary_container
                    } else {
                        self.pal.text
                    };
                    let r = if sel { 12.0 * u } else { 20.0 * u };
                    let style = ts(M3_LABEL_MEDIUM, ink, u);
                    let tw = ui.text_width(lb, style.px, style.medium, style.tracking_em);
                    let bw = 14.0 * u + 20.0 * u + tw + 14.0 * u;
                    let rect = Rect {
                        x: sx,
                        y: ry,
                        w: bw,
                        h: 40.0 * u,
                    };
                    ui.rect(sx, ry, bw, 40.0 * u, fill, r4(r));
                    self.layer(ui, Action::StdGroup(i), rect, r, ink);
                    self.draw_ripples(ui, rect);
                    icon_c(ui, ic, sx + 20.0 * u, ry + 20.0 * u, 18.0 * u, ink);
                    ui.text(lb, sx + 34.0 * u, ry + 20.0 * u - style.line_h * 0.5, style);
                    self.zone(Action::StdGroup(i), rect, ink);
                    sx += bw + 6.0 * u;
                }
            }
            ComponentId::Card => {
                // Show all 3 M3 Card variants side-by-side: Elevated, Filled, Outlined with M3Shape artwork
                let gap = 10.0 * u;
                let cw = (w - gap * 2.0) / 3.0;
                let ch = 172.0 * u;
                let specs = [
                    (
                        "Elevated",
                        "Shadow + SurfaceLow",
                        MaterialShape::Sunny,
                        self.pal.surface_low,
                        None,
                        true,
                        30u32,
                    ),
                    (
                        "Filled",
                        "SurfaceHighest fill",
                        MaterialShape::Clover4Leaf,
                        self.pal.card_highest,
                        None,
                        false,
                        31u32,
                    ),
                    (
                        "Outlined",
                        "1dp crisp border",
                        MaterialShape::Cookie9Sided,
                        self.pal.card,
                        Some(self.pal.outline),
                        false,
                        32u32,
                    ),
                ];
                for (i, (title, desc, shape, fill, border, elevated, bid)) in
                    specs.into_iter().enumerate()
                {
                    let cx = x + i as f32 * (cw + gap);
                    if elevated {
                        let e = self.elev_dp(Action::Button(bid)) * u;
                        ui.rect(
                            cx,
                            y + e,
                            cw,
                            ch,
                            with_alpha(self.pal.bg, 0.7),
                            r4(16.0 * u),
                        );
                    }
                    ui.rect_styled(
                        cx,
                        y,
                        cw,
                        ch,
                        fill,
                        border.unwrap_or([0.0; 4]),
                        if border.is_some() { 1.0 * u } else { 0.0 },
                        r4(16.0 * u),
                        0.0,
                    );
                    // Media header banner inside card with M3Shape
                    ui.push_clip([cx, y, cw, ch]);
                    ui.rect(
                        cx,
                        y,
                        cw,
                        68.0 * u,
                        if i == 1 {
                            self.pal.tertiary_container
                        } else {
                            self.pal.primary_container
                        },
                        [16.0 * u, 16.0 * u, 6.0 * u, 6.0 * u],
                    );
                    ui.m3_shape(
                        cx + cw - 28.0 * u,
                        y + 34.0 * u,
                        44.0 * u,
                        shape,
                        self.t * 0.25 + i as f32,
                        if i == 1 {
                            self.pal.tertiary
                        } else {
                            self.pal.primary
                        },
                        [0.0; 4],
                        0.0,
                    );
                    ui.text(
                        title,
                        cx + 12.0 * u,
                        y + 78.0 * u,
                        ts(M3_TITLE_MEDIUM, self.pal.text, u),
                    );
                    ui.text(
                        desc,
                        cx + 12.0 * u,
                        y + 102.0 * u,
                        ts(M3_BODY_SMALL, self.pal.sub, u),
                    );
                    ui.pop_clip();
                    self.draw_button(
                        ui,
                        bid,
                        cx + 8.0 * u,
                        y + 126.0 * u,
                        BtnSpec {
                            label: "Explore",
                            icon: None,
                            variant: Btn::Tonal,
                        },
                    );
                }
            }
            ComponentId::Checkbox => {
                let labels = [
                    "Checked (Expressive stroke)",
                    "Unchecked",
                    "Indeterminate parent",
                ];
                for (i, label) in labels.into_iter().enumerate() {
                    self.draw_checkbox(ui, i, x, y + i as f32 * 40.0 * u, label);
                }
            }
            ComponentId::Switch => {
                let specs = [
                    ("Adaptive notifications", false),
                    ("Quiet hours (with icons)", true),
                    ("Expressive haptics (with icons)", true),
                ];
                for (i, (label, with_ic)) in specs.into_iter().enumerate() {
                    self.draw_switch(ui, i, x, y + i as f32 * 44.0 * u, label, with_ic);
                }
            }
            ComponentId::RadioButton => {
                let labels = [
                    "Expressive motion (Default)",
                    "Standard motion",
                    "Reduced motion",
                ];
                for (i, label) in labels.into_iter().enumerate() {
                    self.draw_radio(ui, i, x, y + i as f32 * 40.0 * u, label);
                }
            }
            ComponentId::Slider => {
                // Row 1: M3 Expressive Standard Slider (16dp tall track, [8, 2, 2, 8] / [2, 8, 8, 2] corners,
                // 6dp gap around 4x44dp handle, leading volume icon, step ticks & stop indicator)
                let pct = format!("{}%", (self.slider * 100.0).round() as i32);
                ui.text(
                    "Standard Expressive Slider",
                    x,
                    y,
                    ts(M3_LABEL_MEDIUM, self.pal.sub, u),
                );
                let pct_style = ts(M3_LABEL_LARGE, self.pal.primary, u);
                let ptw =
                    ui.text_width(&pct, pct_style.px, pct_style.medium, pct_style.tracking_em);
                ui.text(&pct, x + w - ptw, y, pct_style);

                let ty = y + 40.0 * u;
                let bw = w;
                let pressed = self.slider_drag;
                let tw = if pressed {
                    2.0 * u
                } else {
                    4.0 * u - 2.0 * u * self.slider_thumb.x.clamp(0.0, 1.0)
                };
                let tx = x + tw * 0.5 + (bw - tw) * self.slider;
                let gap = 6.0 * u;
                let aw = (tx - gap - x).max(0.0);
                // Active track with outer 8dp, inner 2dp corners
                if aw > 2.0 * u {
                    ui.rect(
                        x,
                        ty - 8.0 * u,
                        aw,
                        16.0 * u,
                        self.pal.primary,
                        [8.0 * u, 2.0 * u, 2.0 * u, 8.0 * u],
                    );
                    if aw > 28.0 * u {
                        icon_c(
                            ui,
                            "volume_up",
                            x + 14.0 * u,
                            ty,
                            14.0 * u,
                            self.pal.on_primary,
                        );
                    }
                }
                // Inactive track with inner 2dp, outer 8dp corners
                let ix = (tx + gap).min(x + bw);
                let iw = (x + bw - ix).max(0.0);
                if iw > 2.0 * u {
                    ui.rect(
                        ix,
                        ty - 8.0 * u,
                        iw,
                        16.0 * u,
                        self.pal.secondary_container,
                        [2.0 * u, 8.0 * u, 8.0 * u, 2.0 * u],
                    );
                }
                // Stop indicator at trailing end
                let sx = x + bw - 6.0 * u;
                if sx > tx + gap + 2.0 * u {
                    ui.circle(sx, ty, 2.0 * u, self.pal.primary);
                }
                // Vertical 4x44dp handle pill
                ui.rect(
                    tx - tw * 0.5,
                    ty - 22.0 * u,
                    tw,
                    44.0 * u,
                    self.pal.primary,
                    r4(tw * 0.5),
                );

                // Row 2: M3 Expressive Wavy Slider (real GPU sinusoidal wave active track)
                ui.text(
                    "Wavy Expressive Slider (GPU Sinusoidal Wave)",
                    x,
                    y + 70.0 * u,
                    ts(M3_LABEL_MEDIUM, self.pal.sub, u),
                );
                let wty = y + 108.0 * u;
                if aw > 2.0 * u {
                    ui.wavy_line(
                        x,
                        x + aw,
                        wty,
                        4.0 * u,
                        3.2 * u,
                        28.0 * u,
                        -self.t * 5.0,
                        self.pal.primary,
                    );
                }
                if iw > 2.0 * u {
                    ui.rect(
                        ix,
                        wty - 2.0 * u,
                        iw,
                        4.0 * u,
                        self.pal.secondary_container,
                        r4(2.0 * u),
                    );
                }
                if sx > tx + gap + 2.0 * u {
                    ui.circle(sx, wty, 2.0 * u, self.pal.primary);
                }
                ui.rect(
                    tx - tw * 0.5,
                    wty - 20.0 * u,
                    tw,
                    40.0 * u,
                    self.pal.primary,
                    r4(tw * 0.5),
                );

                self.zone(
                    Action::Slider,
                    Rect {
                        x,
                        y: y + 16.0 * u,
                        w,
                        h: 112.0 * u,
                    },
                    self.pal.primary,
                );
            }
            ComponentId::ProgressIndicator => {
                // 1. Determinate LinearWavyProgressIndicator (follows slider)
                let p = self.slider.clamp(0.0, 1.0);
                let stroke_w = 4.0 * u;
                let gap = 4.0 * u;
                let phase = -self.t * 7.5;
                let y1 = y + 8.0 * u;
                let active_w = w * p;
                let amp_factor = (p * 10.0).min((1.0 - p) * 10.0).clamp(0.0, 1.0);
                if active_w > 1.0 * u {
                    ui.wavy_line(
                        x,
                        x + active_w,
                        y1,
                        stroke_w,
                        3.0 * u * amp_factor,
                        40.0 * u,
                        phase,
                        self.pal.primary,
                    );
                }
                let track_x = (x + active_w + gap).min(x + w);
                let track_w = (x + w - track_x).max(0.0);
                if track_w > 1.0 * u {
                    ui.rect(
                        track_x,
                        y1 - stroke_w * 0.5,
                        track_w,
                        stroke_w,
                        self.pal.secondary_container,
                        r4(stroke_w * 0.5),
                    );
                    ui.circle(x + w - stroke_w * 0.5, y1, stroke_w * 0.5, self.pal.primary);
                }

                // 2. Indeterminate LinearWavyProgressIndicator (2-segment cubic-bezier animation)
                let y2 = y + 32.0 * u;
                let cycle = (self.t / 1.8).fract();
                let l1_head = cubic_bezier_y((cycle / 0.65).clamp(0.0, 1.0), 0.3, 0.0, 0.8, 0.15);
                let l1_tail =
                    cubic_bezier_y(((cycle - 0.18) / 0.65).clamp(0.0, 1.0), 0.05, 0.7, 0.1, 1.0);
                let seg_s = x + w * l1_tail;
                let seg_e = x + w * l1_head;
                if seg_s - gap > x + 2.0 * u {
                    ui.rect(
                        x,
                        y2 - stroke_w * 0.5,
                        seg_s - gap - x,
                        stroke_w,
                        self.pal.secondary_container,
                        r4(stroke_w * 0.5),
                    );
                }
                if seg_e > seg_s + 2.0 * u {
                    ui.wavy_line(
                        seg_s,
                        seg_e,
                        y2,
                        stroke_w,
                        3.0 * u,
                        20.0 * u,
                        phase * 1.2,
                        self.pal.primary,
                    );
                }
                if x + w > seg_e + gap + 2.0 * u {
                    ui.rect(
                        seg_e + gap,
                        y2 - stroke_w * 0.5,
                        x + w - (seg_e + gap),
                        stroke_w,
                        self.pal.secondary_container,
                        r4(stroke_w * 0.5),
                    );
                }

                // 3. Flat M3 Expressive LinearProgressIndicator (with gap & stop indicator)
                let y3 = y + 54.0 * u;
                if active_w > 1.0 * u {
                    ui.rect(
                        x,
                        y3 - stroke_w * 0.5,
                        active_w,
                        stroke_w,
                        self.pal.primary,
                        r4(stroke_w * 0.5),
                    );
                }
                if track_w > 1.0 * u {
                    ui.rect(
                        track_x,
                        y3 - stroke_w * 0.5,
                        track_w,
                        stroke_w,
                        self.pal.secondary_container,
                        r4(stroke_w * 0.5),
                    );
                    ui.circle(x + w - stroke_w * 0.5, y3, stroke_w * 0.5, self.pal.primary);
                }

                // 4. Four Circular Indicators (Flat Det, Flat Indet, Wavy Det, Wavy Indet)
                let cy = y + 114.0 * u;
                let r = 24.0 * u;
                let step_x = (w / 4.0).min(96.0 * u);
                let ind_start = (self.t * 220.0) % 360.0;
                let ind_sweep = 60.0 + 210.0 * ((self.t * 2.4).sin() * 0.5 + 0.5);

                let c_specs = [
                    (0, -90.0, p * 360.0, false, "Flat Det"),
                    (1, ind_start, ind_sweep, false, "Flat Indet"),
                    (2, -90.0, p * 360.0, true, "Wavy Det"),
                    (3, ind_start, ind_sweep, true, "Wavy Indet"),
                ];
                for (idx, s_deg, sw_deg, is_wavy, lbl) in c_specs {
                    let cx = x + step_x * (idx as f32 + 0.5);
                    self.draw_circular_progress(ui, cx, cy, r, 4.0 * u, s_deg, sw_deg, is_wavy);
                    text_c(
                        ui,
                        lbl,
                        cx,
                        cy + 28.0 * u,
                        ts(M3_LABEL_SMALL, self.pal.sub, u),
                    );
                }
            }
            ComponentId::LoadingIndicator => {
                // Authentic M3 Expressive LoadingIndicator & ContainedLoadingIndicator
                // cycling through SoftBurst -> Cookie9Sided -> Pentagon -> Pill -> Sunny -> Cookie4Sided -> Oval
                let cy = y + 30.0 * u;

                // 1. Default Uncontained LoadingIndicator (38dp)
                let (sa0, sb0, mt0, rot0, sc0) = loading_morph_state(self.t, 0);
                ui.m3_shape_morph(
                    x + 30.0 * u,
                    cy,
                    38.0 * u * sc0,
                    sa0,
                    sb0,
                    mt0,
                    rot0,
                    self.pal.primary,
                    [0.0; 4],
                    0.0,
                );

                // 2. ContainedLoadingIndicator (48dp primary_container circle + 34dp on_primary_container morph)
                let (sa1, sb1, mt1, rot1, sc1) = loading_morph_state(self.t, 2);
                ui.circle(x + 98.0 * u, cy, 24.0 * u, self.pal.primary_container);
                ui.m3_shape_morph(
                    x + 98.0 * u,
                    cy,
                    34.0 * u * sc1,
                    sa1,
                    sb1,
                    mt1,
                    rot1,
                    self.pal.on_primary_container,
                    [0.0; 4],
                    0.0,
                );

                // 3. Expressive Secondary ContainedLoadingIndicator (54dp SoftBurst container + 36dp morph)
                let (sa2, sb2, mt2, rot2, sc2) = loading_morph_state(self.t, 4);
                ui.m3_shape(
                    x + 170.0 * u,
                    cy,
                    54.0 * u,
                    MaterialShape::Circle,
                    0.0,
                    self.pal.tertiary_container,
                    [0.0; 4],
                    0.0,
                );
                ui.m3_shape_morph(
                    x + 170.0 * u,
                    cy,
                    38.0 * u * sc2,
                    sa2,
                    sb2,
                    mt2,
                    rot2,
                    self.pal.on_tertiary_container,
                    [0.0; 4],
                    0.0,
                );

                ui.text(
                    "7-Polygon Morph",
                    x + 216.0 * u,
                    y + 8.0 * u,
                    ts(M3_TITLE_SMALL, self.pal.text, u),
                );
                ui.text(
                    "SoftBurst -> Cookie9 -> Pentagon -> Pill -> Sunny",
                    x + 216.0 * u,
                    y + 30.0 * u,
                    ts(M3_BODY_SMALL, self.pal.sub, u),
                );

                // Bottom row: Showcase strip of 8 distinct M3 Expressive MaterialShapes rendered via GPU SDF
                let showcase = [
                    (MaterialShape::Sunny, "Sunny"),
                    (MaterialShape::Cookie9Sided, "Cookie9"),
                    (MaterialShape::Clover4Leaf, "Clover4"),
                    (MaterialShape::Heart, "Heart"),
                    (MaterialShape::SoftBurst, "Burst"),
                    (MaterialShape::Flower, "Flower"),
                    (MaterialShape::PuffyDiamond, "Puffy"),
                    (MaterialShape::Ghostish, "Ghost"),
                ];
                let step_w = w / showcase.len() as f32;
                let sy = y + 92.0 * u;
                for (i, (sh, name)) in showcase.into_iter().enumerate() {
                    let sx = x + step_w * (i as f32 + 0.5);
                    let fill = if i % 2 == 0 {
                        self.pal.secondary_container
                    } else {
                        self.pal.primary_container
                    };
                    let stroke = if i % 2 == 0 {
                        self.pal.secondary
                    } else {
                        self.pal.primary
                    };
                    ui.m3_shape(sx, sy, 34.0 * u, sh, 0.0, fill, stroke, 1.2 * u);
                    text_c(
                        ui,
                        name,
                        sx,
                        sy + 20.0 * u,
                        ts(M3_LABEL_SMALL, self.pal.sub, u),
                    );
                }
            }
            ComponentId::AssistChip => {
                let mut cx = x;
                cx += self.draw_chip(
                    ui,
                    0,
                    cx,
                    y + 6.0 * u,
                    "Add to calendar",
                    (Some("calendar_today"), None),
                ) + 8.0 * u;
                self.draw_chip(ui, 0, cx, y + 6.0 * u, "Share file", (Some("share"), None));
            }
            ComponentId::FilterChip => {
                let lead = if self.chips[1] { Some("check") } else { None };
                let mut cx = x;
                cx += self.draw_chip(ui, 1, cx, y + 6.0 * u, "Expressive", (lead, None)) + 8.0 * u;
                self.draw_chip(
                    ui,
                    1,
                    cx,
                    y + 6.0 * u,
                    "Category",
                    (lead, Some("expand_more")),
                );
            }
            ComponentId::InputChip => {
                let mut cx = x;
                cx += self.draw_chip(
                    ui,
                    2,
                    cx,
                    y + 6.0 * u,
                    "alex@geek.dev",
                    (Some("person"), Some("close")),
                ) + 8.0 * u;
                self.draw_chip(
                    ui,
                    2,
                    cx,
                    y + 6.0 * u,
                    "Attachment.pdf",
                    (Some("attach_file"), Some("close")),
                );
            }
            ComponentId::SuggestionChip => {
                let mut cx = x;
                cx +=
                    self.draw_chip(ui, 3, cx, y + 6.0 * u, "Set reminder", (None, None)) + 8.0 * u;
                self.draw_chip(ui, 3, cx, y + 6.0 * u, "Reply all", (Some("reply"), None));
            }
            ComponentId::Dialog => {
                let dw = (340.0 * u).min(w);
                let dx = x + (w - dw) * 0.5;
                let dh = 216.0 * u;
                ui.rect(
                    dx,
                    y + 4.0 * u,
                    dw,
                    dh,
                    with_alpha(self.pal.bg, 0.7),
                    r4(28.0 * u),
                );
                ui.rect_styled(
                    dx,
                    y,
                    dw,
                    dh,
                    self.pal.card_hi,
                    self.pal.outline_variant,
                    1.0 * u,
                    r4(28.0 * u),
                    0.0,
                );
                // M3 Expressive SoftBurst badge behind dialog hero icon
                ui.m3_shape(
                    dx + dw * 0.5,
                    y + 34.0 * u,
                    40.0 * u,
                    MaterialShape::SoftBurst,
                    0.0,
                    self.pal.error_container,
                    [0.0; 4],
                    0.0,
                );
                icon_c(
                    ui,
                    "delete",
                    dx + dw * 0.5,
                    y + 34.0 * u,
                    22.0 * u,
                    self.pal.on_error_container,
                );
                text_c(
                    ui,
                    "Delete project?",
                    dx + dw * 0.5,
                    y + 62.0 * u,
                    ts(M3_HEADLINE_SMALL, self.pal.text, u),
                );
                text_c(
                    ui,
                    "This permanent action removes all repositories",
                    dx + dw * 0.5,
                    y + 100.0 * u,
                    ts(M3_BODY_MEDIUM, self.pal.sub, u),
                );
                text_c(
                    ui,
                    "and build artifacts from this workspace.",
                    dx + dw * 0.5,
                    y + 120.0 * u,
                    ts(M3_BODY_MEDIUM, self.pal.sub, u),
                );
                let st = ts(M3_LABEL_LARGE, self.pal.primary, u);
                let w_c =
                    24.0 * u + ui.text_width("Cancel", st.px, st.medium, st.tracking_em) + 24.0 * u;
                let w_d = 40.0 * u + ui.text_width("Delete", st.px, st.medium, st.tracking_em);
                let bx = dx + dw - w_c - w_d - 24.0 * u;
                self.draw_button(
                    ui,
                    31,
                    bx,
                    y + 160.0 * u,
                    BtnSpec {
                        label: "Cancel",
                        icon: None,
                        variant: Btn::Text,
                    },
                );
                self.draw_button(
                    ui,
                    32,
                    bx + w_c + 8.0 * u,
                    y + 160.0 * u,
                    BtnSpec {
                        label: "Delete",
                        icon: None,
                        variant: Btn::Filled,
                    },
                );
            }
            ComponentId::BottomSheet => {
                ui.rect(
                    x,
                    y,
                    w,
                    184.0 * u,
                    self.pal.card_hi,
                    [28.0 * u, 28.0 * u, 12.0 * u, 12.0 * u],
                );
                // Drag handle (32x4dp)
                ui.rect(
                    x + w * 0.5 - 16.0 * u,
                    y + 12.0 * u,
                    32.0 * u,
                    4.0 * u,
                    self.pal.outline,
                    r4(2.0 * u),
                );
                let items = [
                    ("share", "Share with collaborators"),
                    ("link", "Copy public link"),
                    ("archive", "Archive project"),
                ];
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let iy = y + 32.0 * u + i as f32 * 48.0 * u;
                    let a = Action::Button(70 + i as u32);
                    let rect = Rect {
                        x: x + 8.0 * u,
                        y: iy,
                        w: w - 16.0 * u,
                        h: 44.0 * u,
                    };
                    self.layer(ui, a, rect, 12.0 * u, self.pal.text);
                    self.draw_ripples(ui, rect);
                    icon_c(
                        ui,
                        ic,
                        x + 28.0 * u,
                        iy + 22.0 * u,
                        22.0 * u,
                        self.pal.primary,
                    );
                    ui.text(
                        lb,
                        x + 56.0 * u,
                        iy + 10.0 * u,
                        ts(M3_BODY_LARGE, self.pal.text, u),
                    );
                    self.zone(a, rect, self.pal.text);
                }
            }
            ComponentId::NavigationBar => {
                let items = [
                    ("home", "Home"),
                    ("explore", "Browse"),
                    ("favorite", "Saved"),
                    ("person", "Profile"),
                ];
                let cw = w / items.len() as f32;
                ui.rect(x, y, w, 80.0 * u, self.pal.card_hi, r4(16.0 * u));
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let cx0 = x + i as f32 * cw;
                    let sel = self.nav == i;
                    let ink = if sel {
                        self.pal.on_secondary_container
                    } else {
                        self.pal.sub
                    };
                    if sel {
                        ui.rect(
                            cx0 + cw * 0.5 - 32.0 * u,
                            y + 12.0 * u,
                            64.0 * u,
                            32.0 * u,
                            self.pal.secondary_container,
                            r4(16.0 * u),
                        );
                    }
                    let rect = Rect {
                        x: cx0,
                        y,
                        w: cw,
                        h: 80.0 * u,
                    };
                    self.layer(ui, Action::Nav(i), rect, 16.0 * u, self.pal.text);
                    self.draw_ripples(ui, rect);
                    icon_c(ui, ic, cx0 + cw * 0.5, y + 28.0 * u, 24.0 * u, ink);
                    let style = ts(
                        M3_LABEL_MEDIUM,
                        if sel { self.pal.text } else { self.pal.sub },
                        u,
                    );
                    text_c(ui, lb, cx0 + cw * 0.5, y + 50.0 * u, style);
                    self.zone(Action::Nav(i), rect, ink);
                }
            }
            ComponentId::NavigationRail => {
                ui.rect(
                    x,
                    y,
                    88.0 * u,
                    232.0 * u,
                    self.pal.surface_low,
                    r4(16.0 * u),
                );
                let items = [
                    ("home", "Home"),
                    ("search", "Search"),
                    ("bookmark", "Saved"),
                    ("person", "Profile"),
                ];
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let iy = y + 8.0 * u + i as f32 * 54.0 * u;
                    let sel = self.rail == i;
                    let ink = if sel {
                        self.pal.on_secondary_container
                    } else {
                        self.pal.sub
                    };
                    if sel {
                        ui.rect(
                            x + 16.0 * u,
                            iy + 4.0 * u,
                            56.0 * u,
                            32.0 * u,
                            self.pal.secondary_container,
                            r4(16.0 * u),
                        );
                    }
                    let rect = Rect {
                        x: x + 4.0 * u,
                        y: iy,
                        w: 80.0 * u,
                        h: 52.0 * u,
                    };
                    self.layer(ui, Action::Rail(i), rect, 16.0 * u, self.pal.text);
                    self.draw_ripples(ui, rect);
                    icon_c(ui, ic, x + 44.0 * u, iy + 20.0 * u, 22.0 * u, ink);
                    let style = ts(
                        M3_LABEL_MEDIUM,
                        if sel { self.pal.text } else { self.pal.sub },
                        u,
                    );
                    text_c(ui, lb, x + 44.0 * u, iy + 36.0 * u, style);
                    self.zone(Action::Rail(i), rect, ink);
                }
                // Content preview pane next to rail
                let px = x + 100.0 * u;
                let pw = (w - 100.0 * u).max(80.0 * u);
                ui.rect(px, y, pw, 232.0 * u, self.pal.card_hi, r4(16.0 * u));
                ui.text(
                    "WideNavigationRail Destination",
                    px + 16.0 * u,
                    y + 18.0 * u,
                    ts(M3_TITLE_MEDIUM, self.pal.text, u),
                );
                ui.text(
                    "Persistent side rail for tablet & desktop layouts.",
                    px + 16.0 * u,
                    y + 46.0 * u,
                    ts(M3_BODY_SMALL, self.pal.sub, u),
                );
            }
            ComponentId::NavigationDrawer => {
                let dw = (280.0 * u).min(w);
                ui.rect(x, y, dw, 288.0 * u, self.pal.card_hi, r4(16.0 * u));
                ui.text(
                    "Expressive Mail",
                    x + 20.0 * u,
                    y + 16.0 * u,
                    ts(M3_TITLE_MEDIUM, self.pal.text, u),
                );
                let items = [
                    ("mail", "Inbox", "24"),
                    ("send", "Outbox", ""),
                    ("favorite", "Favorites", "5"),
                    ("delete", "Trash", ""),
                ];
                for (i, (ic, lb, badge)) in items.into_iter().enumerate() {
                    let iy = y + 52.0 * u + i as f32 * 56.0 * u;
                    let sel = self.drawer == i;
                    let ink = if sel {
                        self.pal.on_secondary_container
                    } else {
                        self.pal.text
                    };
                    let rect = Rect {
                        x: x + 12.0 * u,
                        y: iy,
                        w: dw - 24.0 * u,
                        h: 52.0 * u,
                    };
                    if sel {
                        ui.rect(
                            rect.x,
                            rect.y,
                            rect.w,
                            rect.h,
                            self.pal.secondary_container,
                            r4(26.0 * u),
                        );
                    }
                    self.layer(ui, Action::Drawer(i), rect, 26.0 * u, self.pal.text);
                    self.draw_ripples(ui, rect);
                    icon_c(ui, ic, x + 32.0 * u, iy + 26.0 * u, 22.0 * u, ink);
                    ui.text(lb, x + 56.0 * u, iy + 16.0 * u, ts(M3_LABEL_LARGE, ink, u));
                    if !badge.is_empty() {
                        let st = ts(M3_LABEL_MEDIUM, ink, u);
                        let btw = ui.text_width(badge, st.px, st.medium, st.tracking_em);
                        ui.text(badge, x + dw - 28.0 * u - btw, iy + 18.0 * u, st);
                    }
                    self.zone(Action::Drawer(i), rect, ink);
                }
            }
            ComponentId::Scaffold => {
                let sw = (340.0 * u).min(w);
                ui.rect_styled(
                    x,
                    y,
                    sw,
                    232.0 * u,
                    self.pal.surface_low,
                    self.pal.outline_variant,
                    1.0 * u,
                    r4(16.0 * u),
                    0.0,
                );
                ui.push_clip([x, y, sw, 232.0 * u]);
                ui.rect(x, y, sw, 52.0 * u, self.pal.card_hi, r4(0.0));
                ui.text(
                    "Scaffold Layout",
                    x + 16.0 * u,
                    y + 14.0 * u,
                    ts(M3_TITLE_MEDIUM, self.pal.text, u),
                );
                icon_c(
                    ui,
                    "more_vert",
                    x + sw - 24.0 * u,
                    y + 26.0 * u,
                    20.0 * u,
                    self.pal.sub,
                );
                ui.rect(
                    x + 16.0 * u,
                    y + 66.0 * u,
                    sw - 32.0 * u,
                    40.0 * u,
                    self.pal.card_hi,
                    r4(10.0 * u),
                );
                ui.text(
                    "TopAppBar + Content + FAB + NavigationBar",
                    x + 28.0 * u,
                    y + 78.0 * u,
                    ts(M3_BODY_SMALL, self.pal.sub, u),
                );
                self.fab_styled(
                    ui,
                    Action::Button(80),
                    x + sw - 64.0 * u,
                    y + 114.0 * u,
                    48.0 * u,
                    14.0 * u,
                    self.pal.primary_container,
                    self.pal.on_primary_container,
                    "add",
                    22.0 * u,
                );
                ui.rect(x, y + 176.0 * u, sw, 56.0 * u, self.pal.card_hi, r4(0.0));
                let icons = ["home", "search", "person"];
                for (i, ic) in icons.into_iter().enumerate() {
                    let cx = x + sw * ((i as f32 + 0.5) / 3.0);
                    if i == 0 {
                        ui.rect(
                            cx - 24.0 * u,
                            y + 188.0 * u,
                            48.0 * u,
                            28.0 * u,
                            self.pal.secondary_container,
                            r4(14.0 * u),
                        );
                    }
                    icon_c(
                        ui,
                        ic,
                        cx,
                        y + 202.0 * u,
                        20.0 * u,
                        if i == 0 {
                            self.pal.on_secondary_container
                        } else {
                            self.pal.sub
                        },
                    );
                }
                ui.pop_clip();
            }
            ComponentId::TopAppBar => {
                // MediumFlexibleTopAppBar with subtitle and tonal actions
                ui.rect(x, y, w, 100.0 * u, self.pal.card_hi, r4(16.0 * u));
                self.icon_button(
                    ui,
                    90,
                    x + 6.0 * u,
                    y + 8.0 * u,
                    "arrow_back",
                    IconBtn::Standard,
                );
                self.icon_button(
                    ui,
                    91,
                    x + w - 92.0 * u,
                    y + 8.0 * u,
                    "attach_file",
                    IconBtn::Tonal,
                );
                self.icon_button(
                    ui,
                    92,
                    x + w - 46.0 * u,
                    y + 8.0 * u,
                    "more_vert",
                    IconBtn::Standard,
                );
                ui.text(
                    "Expressive Catalog",
                    x + 16.0 * u,
                    y + 50.0 * u,
                    ts(M3_HEADLINE_SMALL, self.pal.text, u),
                );
                ui.text(
                    "MediumFlexibleTopAppBar with subtitle",
                    x + 16.0 * u,
                    y + 78.0 * u,
                    ts(M3_LABEL_MEDIUM, self.pal.sub, u),
                );
            }
            ComponentId::SearchBar => {
                let sw = (380.0 * u).min(w);
                ui.rect(
                    x,
                    y + 2.0 * u,
                    sw,
                    56.0 * u,
                    with_alpha(self.pal.bg, 0.65),
                    r4(28.0 * u),
                );
                ui.rect(x, y, sw, 56.0 * u, self.pal.card_hi, r4(28.0 * u));
                let a = Action::Button(93);
                let rect = Rect {
                    x,
                    y,
                    w: sw,
                    h: 56.0 * u,
                };
                self.layer(ui, a, rect, 28.0 * u, self.pal.text);
                self.draw_ripples(ui, rect);
                icon_c(
                    ui,
                    "search",
                    x + 24.0 * u,
                    y + 28.0 * u,
                    22.0 * u,
                    self.pal.text,
                );
                ui.text(
                    "Search M3 Expressive components...",
                    x + 52.0 * u,
                    y + 16.0 * u,
                    ts(M3_BODY_LARGE, self.pal.sub, u),
                );
                // Trailing Clover4Leaf avatar + mic icon
                icon_c(
                    ui,
                    "mic",
                    x + sw - 64.0 * u,
                    y + 28.0 * u,
                    20.0 * u,
                    self.pal.sub,
                );
                ui.m3_shape(
                    x + sw - 28.0 * u,
                    y + 28.0 * u,
                    32.0 * u,
                    MaterialShape::Clover4Leaf,
                    0.0,
                    self.pal.primary_container,
                    [0.0; 4],
                    0.0,
                );
                icon_c(
                    ui,
                    "person",
                    x + sw - 28.0 * u,
                    y + 28.0 * u,
                    18.0 * u,
                    self.pal.on_primary_container,
                );
                self.zone(a, rect, self.pal.text);
            }
            ComponentId::TextField => {
                // 1. Filled TextField
                let f = self.focus.x.clamp(0.0, 1.0);
                ui.rect(
                    x,
                    y,
                    w,
                    56.0 * u,
                    self.pal.card_hi,
                    [4.0 * u, 4.0 * u, 0.0, 0.0],
                );
                icon_c(
                    ui,
                    "mail",
                    x + 24.0 * u,
                    y + 28.0 * u,
                    20.0 * u,
                    self.pal.sub,
                );
                ui.text(
                    "Email address",
                    x + 48.0 * u,
                    y + 8.0 * u,
                    ts(M3_BODY_SMALL, mix(self.pal.sub, self.pal.primary, f), u),
                );
                ui.text(
                    "material.expressive@google.com",
                    x + 48.0 * u,
                    y + 26.0 * u,
                    ts(M3_BODY_LARGE, self.pal.text, u),
                );
                let ih = (1.0 + f) * u;
                ui.rect(
                    x,
                    y + 56.0 * u - ih,
                    w,
                    ih,
                    mix(self.pal.outline, self.pal.primary, f),
                    r4(0.0),
                );

                // 2. OutlinedTextField with genuine floating-label cutout on the top border
                let oy = y + 80.0 * u;
                ui.stroke(x, oy, w, 56.0 * u, self.pal.primary, r4(4.0 * u), 1.5 * u);
                let lbl = "Password";
                let lbl_style = ts(M3_BODY_SMALL, self.pal.primary, u);
                let ltw = ui.text_width(lbl, lbl_style.px, lbl_style.medium, lbl_style.tracking_em);
                // Cutout mask over the top border so the border never slices through the label
                ui.rect(
                    x + 12.0 * u,
                    oy - 4.0 * u,
                    ltw + 8.0 * u,
                    8.0 * u,
                    self.pal.card,
                    r4(0.0),
                );
                ui.text(lbl, x + 16.0 * u, oy - 8.0 * u, lbl_style);
                icon_c(
                    ui,
                    "lock",
                    x + 24.0 * u,
                    oy + 28.0 * u,
                    20.0 * u,
                    self.pal.primary,
                );
                ui.text(
                    "••••••••••••",
                    x + 48.0 * u,
                    oy + 18.0 * u,
                    ts(M3_BODY_LARGE, self.pal.text, u),
                );
                icon_c(
                    ui,
                    "visibility_off",
                    x + w - 24.0 * u,
                    oy + 28.0 * u,
                    20.0 * u,
                    self.pal.sub,
                );
            }
            ComponentId::Menu => {
                // M3 Expressive DropdownMenuPopup with grouped rounded sections (16dp outer, 4dp inner, 6dp group gap)
                let mw = (260.0 * u).min(w);
                let items = [
                    ("edit", "Rename file", "Ctrl+R"),
                    ("share", "Share link", "Ctrl+S"),
                    ("archive", "Move to archive", ""),
                    ("delete", "Delete permanently", "Del"),
                ];
                let mut iy = y;
                for (i, (ic, lb, shortcut)) in items.into_iter().enumerate() {
                    let sel = self.menu_sel == i;
                    let fill = if sel {
                        self.pal.secondary_container
                    } else {
                        self.pal.card_hi
                    };
                    let ink = if sel {
                        self.pal.on_secondary_container
                    } else if i == 3 {
                        self.pal.error
                    } else {
                        self.pal.text
                    };
                    // Group 1 = items 0..=2, Group 2 = item 3 (destructive)
                    let radii = match i {
                        0 => [16.0 * u, 16.0 * u, 4.0 * u, 4.0 * u],
                        1 => r4(4.0 * u),
                        2 => [4.0 * u, 4.0 * u, 16.0 * u, 16.0 * u],
                        _ => r4(16.0 * u),
                    };
                    let rect = Rect {
                        x,
                        y: iy,
                        w: mw,
                        h: 48.0 * u,
                    };
                    ui.rect(rect.x, rect.y, rect.w, rect.h, fill, radii);
                    self.layer_radii(ui, Action::MenuItem(i), rect, radii, ink);
                    self.draw_ripples(ui, rect);
                    icon_c(ui, ic, x + 22.0 * u, iy + 24.0 * u, 20.0 * u, ink);
                    ui.text(lb, x + 46.0 * u, iy + 14.0 * u, ts(M3_BODY_LARGE, ink, u));
                    if sel {
                        icon_c(ui, "check", x + mw - 22.0 * u, iy + 24.0 * u, 18.0 * u, ink);
                    } else if !shortcut.is_empty() {
                        let st = ts(M3_LABEL_SMALL, self.pal.sub, u);
                        let stw = ui.text_width(shortcut, st.px, st.medium, st.tracking_em);
                        ui.text(shortcut, x + mw - 16.0 * u - stw, iy + 16.0 * u, st);
                    }
                    self.zone(Action::MenuItem(i), rect, ink);
                    iy += if i == 2 { 56.0 * u } else { 50.0 * u };
                }
            }
            ComponentId::Carousel => {
                // Authentic M3 Expressive Carousel (Compose CarouselSamples.kt port):
                // 1. Mode selector pills: Multi-browse | Centered hero | Uncontained
                let modes = ["Multi-browse", "Centered hero", "Uncontained"];
                let mut mx = x;
                for (m, label) in modes.into_iter().enumerate() {
                    let sel = self.carousel_mode == m;
                    let fill = if sel {
                        self.pal.primary
                    } else {
                        self.pal.card_hi
                    };
                    let ink = if sel {
                        self.pal.on_primary
                    } else {
                        self.pal.text
                    };
                    let st = ts(M3_LABEL_MEDIUM, ink, u);
                    let tw = ui.text_width(label, st.px, st.medium, st.tracking_em);
                    let mw = tw + 24.0 * u;
                    let m_rect = Rect {
                        x: mx,
                        y,
                        w: mw,
                        h: 28.0 * u,
                    };
                    ui.rect(mx, y, mw, 28.0 * u, fill, r4(14.0 * u));
                    self.layer(ui, Action::CarouselMode(m), m_rect, 14.0 * u, ink);
                    ui.text(label, mx + 12.0 * u, y + 6.0 * u, st);
                    self.zone(Action::CarouselMode(m), m_rect, ink);
                    mx += mw + 8.0 * u;
                }

                // 2. Carousel Track (Uniform 176dp height, 28dp corner radius, dynamic width squishing!)
                let cy = y + 38.0 * u;
                let ch = 176.0 * u;
                let gap = 8.0 * u;
                #[allow(clippy::type_complexity)]
                let items: [(&str, &str, &str, MaterialShape, [f32; 4], [f32; 4]);
                    5] = [
                    (
                        "Alpine Dawn",
                        "Dolomites • 4K HDR",
                        "landscape",
                        MaterialShape::Sunny,
                        argb(0x3B236A),
                        argb(0xD0BCFF),
                    ),
                    (
                        "Pacific Swell",
                        "Big Sur Coastline",
                        "waves",
                        MaterialShape::Clover4Leaf,
                        argb(0x1E3F5A),
                        argb(0x9CCAFF),
                    ),
                    (
                        "Redwood Mist",
                        "Humboldt Reserve",
                        "park",
                        MaterialShape::Flower,
                        argb(0x214738),
                        argb(0xA6F2D1),
                    ),
                    (
                        "Sahara Gold",
                        "Erg Chebbi Dunes",
                        "sunny",
                        MaterialShape::SoftBurst,
                        argb(0x5B3224),
                        argb(0xFFB59A),
                    ),
                    (
                        "Kyoto Bloom",
                        "Arashiyama Grove",
                        "local_cafe",
                        MaterialShape::Cookie9Sided,
                        argb(0x592642),
                        argb(0xFFD8E8),
                    ),
                ];
                let n = items.len();
                let focus = self.carousel_pos.x.clamp(0.0, (n - 1) as f32);

                // Compute dynamic M3E item widths based on mode & continuous spring focus
                let mut raw_weights = [0.0f32; 5];
                for (i, slot) in raw_weights.iter_mut().enumerate() {
                    let dist = (i as f32 - focus).abs();
                    *slot = match self.carousel_mode {
                        0 => {
                            // HorizontalMultiBrowseCarousel: Large -> Medium -> Small -> Peek
                            if dist < 1.0 {
                                lerp(4.2, 2.2, dist)
                            } else if dist < 2.0 {
                                lerp(2.2, 0.95, dist - 1.0)
                            } else {
                                lerp(0.95, 0.65, (dist - 2.0).min(1.0))
                            }
                        }
                        1 => {
                            // HorizontalCenteredHeroCarousel: Small | HERO | Small
                            if dist < 1.0 {
                                lerp(4.6, 1.05, dist)
                            } else {
                                lerp(1.05, 0.55, (dist - 1.0).min(1.0))
                            }
                        }
                        _ => {
                            // HorizontalUncontainedCarousel: larger uniform cards with trailing peek
                            if dist < 1.5 {
                                lerp(3.2, 2.6, dist / 1.5)
                            } else {
                                1.15
                            }
                        }
                    };
                }
                let weight_sum: f32 = raw_weights.iter().sum();
                let avail_w = (w - gap * (n as f32 - 1.0)).max(120.0 * u);
                let mut cx0 = x;
                for (i, (title, sub, ic, shape, bg_col, accent_col)) in
                    items.into_iter().enumerate()
                {
                    let cw = avail_w * (raw_weights[i] / weight_sum);
                    // 28dp corner radius (MaterialTheme.shapes.extraLarge), clamped to cw * 0.5 for narrow peek pills
                    let r = (28.0 * u).min(cw * 0.5);
                    let card_rect = Rect {
                        x: cx0,
                        y: cy,
                        w: cw,
                        h: ch,
                    };

                    // Base card fill
                    ui.rect(cx0, cy, cw, ch, bg_col, r4(r));

                    // Clip all rich artwork and labels cleanly inside the card bounds
                    ui.push_clip([cx0, cy, cw, ch]);

                    // Decorative M3 Expressive shape artwork in upper right of card
                    ui.m3_shape(
                        cx0 + cw * 0.68,
                        cy + ch * 0.36,
                        (cw * 0.78).min(124.0 * u).max(44.0 * u),
                        shape,
                        0.18 * i as f32 + self.t * 0.08,
                        with_alpha(accent_col, 0.24),
                        with_alpha(accent_col, 0.45),
                        1.5 * u,
                    );
                    // Secondary ambient circle/pill accent
                    ui.circle(
                        cx0 + cw * 0.22,
                        cy + ch * 0.24,
                        28.0 * u,
                        with_alpha(accent_col, 0.14),
                    );

                    // Bottom gradient-like scrim for high-contrast typography
                    ui.rect(
                        cx0,
                        cy + ch - 68.0 * u,
                        cw,
                        68.0 * u,
                        with_alpha(self.pal.bg, 0.62),
                        [0.0, 0.0, r, r],
                    );

                    // Top-left icon pill badge
                    ui.circle(
                        cx0 + (24.0 * u).min(cw * 0.5),
                        cy + 24.0 * u,
                        14.0 * u,
                        with_alpha(self.pal.bg, 0.45),
                    );
                    icon_c(
                        ui,
                        ic,
                        cx0 + (24.0 * u).min(cw * 0.5),
                        cy + 24.0 * u,
                        16.0 * u,
                        accent_col,
                    );

                    // Reveal title & subtitle smoothly when the card is wide enough
                    let text_alpha = ((cw - 76.0 * u) / (44.0 * u)).clamp(0.0, 1.0);
                    if text_alpha > 0.05 {
                        ui.text(
                            title,
                            cx0 + 14.0 * u,
                            cy + ch - 52.0 * u,
                            ts(M3_TITLE_MEDIUM, with_alpha(self.pal.text, text_alpha), u),
                        );
                        ui.text(
                            sub,
                            cx0 + 14.0 * u,
                            cy + ch - 28.0 * u,
                            ts(M3_BODY_SMALL, with_alpha(accent_col, text_alpha), u),
                        );
                    }

                    ui.pop_clip();

                    // Crisp 1dp outline border around the 28dp card
                    ui.stroke(
                        cx0,
                        cy,
                        cw,
                        ch,
                        with_alpha(accent_col, if self.carousel == i { 0.65 } else { 0.22 }),
                        r4(r),
                        1.0 * u,
                    );

                    self.layer(ui, Action::CarouselPage(i), card_rect, r, self.pal.text);
                    self.draw_ripples(ui, card_rect);
                    self.zone(Action::CarouselPage(i), card_rect, self.pal.text);
                    cx0 += cw + gap;
                }
            }
            ComponentId::DatePicker => {
                ui.text(
                    "Select date • Oct 2026",
                    x + 12.0 * u,
                    y + 6.0 * u,
                    ts(M3_TITLE_MEDIUM, self.pal.text, u),
                );
                icon_c(
                    ui,
                    "arrow_back",
                    x + w - 64.0 * u,
                    y + 18.0 * u,
                    20.0 * u,
                    self.pal.sub,
                );
                icon_c(
                    ui,
                    "arrow_forward",
                    x + w - 28.0 * u,
                    y + 18.0 * u,
                    20.0 * u,
                    self.pal.sub,
                );
                let cw = (w / 7.0).min(46.0 * u);
                let x0 = x + (w - 7.0 * cw) * 0.5;
                let wd = ["S", "M", "T", "W", "T", "F", "S"];
                for (i, d) in wd.into_iter().enumerate() {
                    text_c(
                        ui,
                        d,
                        x0 + i as f32 * cw + cw * 0.5,
                        y + 40.0 * u,
                        ts(M3_LABEL_MEDIUM, self.pal.sub, u),
                    );
                }
                for day in 0..31 {
                    let slot = day + 4; // Oct 1, 2026 starts on Thursday (index 4)
                    let col = slot % 7;
                    let row = slot / 7;
                    let gx = x0 + col as f32 * cw + cw * 0.5;
                    let gy = y + 68.0 * u + row as f32 * 40.0 * u + 20.0 * u;
                    let sel = day == self.date_day;
                    let today = day == 15;
                    let hit = Rect {
                        x: gx - 18.0 * u,
                        y: gy - 18.0 * u,
                        w: 36.0 * u,
                        h: 36.0 * u,
                    };
                    if sel {
                        ui.circle(gx, gy, 18.0 * u, self.pal.primary);
                    } else if today {
                        ui.stroke(
                            hit.x,
                            hit.y,
                            hit.w,
                            hit.h,
                            self.pal.primary,
                            r4(18.0 * u),
                            1.2 * u,
                        );
                    }
                    let label = format!("{}", day + 1);
                    text_c(
                        ui,
                        &label,
                        gx,
                        gy - 10.0 * u,
                        ts(
                            M3_LABEL_LARGE,
                            if sel {
                                self.pal.on_primary
                            } else if today {
                                self.pal.primary
                            } else {
                                self.pal.text
                            },
                            u,
                        ),
                    );
                    self.zone(Action::DateDay(day), hit, self.pal.primary);
                }
            }
            ComponentId::TimePicker => {
                // Left side: M3 Digital Time Input boxes (10 : 24 + AM/PM toggle)
                ui.text("Select time", x, y, ts(M3_LABEL_MEDIUM, self.pal.sub, u));
                // Hour box (selected)
                ui.rect(
                    x,
                    y + 24.0 * u,
                    68.0 * u,
                    56.0 * u,
                    self.pal.primary_container,
                    r4(12.0 * u),
                );
                ui.stroke(
                    x,
                    y + 24.0 * u,
                    68.0 * u,
                    56.0 * u,
                    self.pal.primary,
                    r4(12.0 * u),
                    2.0 * u,
                );
                text_c(
                    ui,
                    "10",
                    x + 34.0 * u,
                    y + 30.0 * u,
                    ts(M3_DISPLAY_SMALL, self.pal.on_primary_container, u),
                );
                text_c(
                    ui,
                    ":",
                    x + 76.0 * u,
                    y + 28.0 * u,
                    ts(M3_DISPLAY_SMALL, self.pal.text, u),
                );
                // Minute box
                ui.rect(
                    x + 84.0 * u,
                    y + 24.0 * u,
                    68.0 * u,
                    56.0 * u,
                    self.pal.card_highest,
                    r4(12.0 * u),
                );
                text_c(
                    ui,
                    "24",
                    x + 118.0 * u,
                    y + 30.0 * u,
                    ts(M3_DISPLAY_SMALL, self.pal.text, u),
                );

                // Interactive AM / PM segmented toggle
                let am_rect = Rect {
                    x,
                    y: y + 92.0 * u,
                    w: 74.0 * u,
                    h: 34.0 * u,
                };
                let pm_rect = Rect {
                    x: x + 78.0 * u,
                    y: y + 92.0 * u,
                    w: 74.0 * u,
                    h: 34.0 * u,
                };
                ui.rect(
                    am_rect.x,
                    am_rect.y,
                    am_rect.w,
                    am_rect.h,
                    if self.time_am {
                        self.pal.tertiary_container
                    } else {
                        self.pal.card_hi
                    },
                    [10.0 * u, 4.0 * u, 4.0 * u, 10.0 * u],
                );
                text_c(
                    ui,
                    "AM",
                    am_rect.x + am_rect.w * 0.5,
                    am_rect.y + 7.0 * u,
                    ts(
                        M3_LABEL_LARGE,
                        if self.time_am {
                            self.pal.on_tertiary_container
                        } else {
                            self.pal.sub
                        },
                        u,
                    ),
                );
                self.zone(Action::TimeAmPm(true), am_rect, self.pal.text);

                ui.rect(
                    pm_rect.x,
                    pm_rect.y,
                    pm_rect.w,
                    pm_rect.h,
                    if !self.time_am {
                        self.pal.tertiary_container
                    } else {
                        self.pal.card_hi
                    },
                    [4.0 * u, 10.0 * u, 10.0 * u, 4.0 * u],
                );
                text_c(
                    ui,
                    "PM",
                    pm_rect.x + pm_rect.w * 0.5,
                    pm_rect.y + 7.0 * u,
                    ts(
                        M3_LABEL_LARGE,
                        if !self.time_am {
                            self.pal.on_tertiary_container
                        } else {
                            self.pal.sub
                        },
                        u,
                    ),
                );
                self.zone(Action::TimeAmPm(false), pm_rect, self.pal.text);

                // Right side: Authentic 12-hour Clock Dial with selector circle on 10
                let cx = x + (w - 102.0 * u).max(248.0 * u);
                let cy = y + 104.0 * u;
                let dial_r = 96.0 * u;
                ui.circle(cx, cy, dial_r, self.pal.card_highest);
                let num_r = 72.0 * u;
                // Selected hour = 10 -> angle = (10 * 30 - 90) deg = 210 deg
                let sel_ang = 210.0f32.to_radians();
                let sel_x = cx + num_r * sel_ang.cos();
                let sel_y = cy + num_r * sel_ang.sin();
                ui.stroke_line([cx, cy], [sel_x, sel_y], 2.5 * u, self.pal.primary);
                ui.circle(cx, cy, 4.5 * u, self.pal.primary);
                ui.circle(sel_x, sel_y, 18.0 * u, self.pal.primary);

                for h_idx in 1..=12 {
                    let deg = h_idx as f32 * 30.0 - 90.0;
                    let ang = deg.to_radians();
                    let nx = cx + num_r * ang.cos();
                    let ny = cy + num_r * ang.sin();
                    let lbl = format!("{h_idx}");
                    text_c(
                        ui,
                        &lbl,
                        nx,
                        ny - 10.0 * u,
                        ts(
                            M3_LABEL_LARGE,
                            if h_idx == 10 {
                                self.pal.on_primary
                            } else {
                                self.pal.text
                            },
                            u,
                        ),
                    );
                }
            }
            ComponentId::Tooltip => {
                // PlainTooltip + RichTooltip with action button
                ui.rect(
                    x + 16.0 * u,
                    y,
                    148.0 * u,
                    28.0 * u,
                    self.pal.inverse,
                    r4(6.0 * u),
                );
                text_c(
                    ui,
                    "Plain tooltip hint",
                    x + 90.0 * u,
                    y + 6.0 * u,
                    ts(M3_BODY_SMALL, self.pal.on_inverse, u),
                );

                let ry = y + 36.0 * u;
                let rw = (320.0 * u).min(w);
                ui.rect_styled(
                    x + 16.0 * u,
                    ry,
                    rw,
                    84.0 * u,
                    self.pal.card_hi,
                    self.pal.outline_variant,
                    1.0 * u,
                    r4(12.0 * u),
                    0.0,
                );
                ui.text(
                    "Rich Expressive Tooltip",
                    x + 32.0 * u,
                    ry + 10.0 * u,
                    ts(M3_TITLE_SMALL, self.pal.text, u),
                );
                ui.text(
                    "Provides contextual details and an inline action.",
                    x + 32.0 * u,
                    ry + 32.0 * u,
                    ts(M3_BODY_SMALL, self.pal.sub, u),
                );
                ui.text(
                    "Learn more",
                    x + 32.0 * u,
                    ry + 56.0 * u,
                    ts(M3_LABEL_LARGE, self.pal.primary, u),
                );
            }
            ComponentId::Snackbar => {
                ui.rect(x, y + 8.0 * u, w, 48.0 * u, self.pal.inverse, r4(8.0 * u));
                let st = ts(M3_LABEL_LARGE, self.pal.inverse_primary, u);
                let tw = ui.text_width("Undo", st.px, st.medium, st.tracking_em);
                let a = Action::Button(96);
                let rect = Rect {
                    x: x + w - 48.0 * u - tw - 16.0 * u,
                    y: y + 8.0 * u,
                    w: tw + 16.0 * u,
                    h: 48.0 * u,
                };
                self.layer(ui, a, rect, 8.0 * u, self.pal.inverse_primary);
                self.draw_ripples(ui, rect);
                ui.text(
                    "Project archived to workspace",
                    x + 16.0 * u,
                    y + 22.0 * u,
                    ts(M3_BODY_MEDIUM, self.pal.on_inverse, u),
                );
                ui.text("Undo", rect.x + 8.0 * u, y + 22.0 * u, st);
                icon_c(
                    ui,
                    "close",
                    x + w - 24.0 * u,
                    y + 32.0 * u,
                    18.0 * u,
                    self.pal.on_inverse,
                );
                self.zone(a, rect, self.pal.inverse_primary);
            }
            ComponentId::Badge => {
                // Small 6dp badge + Single-digit 16dp badge + Multi-digit "99+" pill badge
                ui.rect(
                    x + 4.0 * u,
                    y + 8.0 * u,
                    40.0 * u,
                    40.0 * u,
                    self.pal.card_hi,
                    r4(20.0 * u),
                );
                icon_c(
                    ui,
                    "notifications",
                    x + 24.0 * u,
                    y + 28.0 * u,
                    22.0 * u,
                    self.pal.text,
                );
                ui.circle(x + 36.0 * u, y + 14.0 * u, 4.0 * u, self.pal.error);

                ui.rect(
                    x + 60.0 * u,
                    y + 8.0 * u,
                    40.0 * u,
                    40.0 * u,
                    self.pal.card_hi,
                    r4(20.0 * u),
                );
                icon_c(
                    ui,
                    "mail",
                    x + 80.0 * u,
                    y + 28.0 * u,
                    22.0 * u,
                    self.pal.text,
                );
                ui.circle(x + 94.0 * u, y + 14.0 * u, 9.0 * u, self.pal.error);
                text_c(
                    ui,
                    "3",
                    x + 94.0 * u,
                    y + 6.0 * u,
                    ts(M3_LABEL_SMALL, self.pal.on_error, u),
                );

                ui.rect(
                    x + 116.0 * u,
                    y + 8.0 * u,
                    40.0 * u,
                    40.0 * u,
                    self.pal.card_hi,
                    r4(20.0 * u),
                );
                icon_c(
                    ui,
                    "folder",
                    x + 136.0 * u,
                    y + 28.0 * u,
                    22.0 * u,
                    self.pal.text,
                );
                ui.rect(
                    x + 142.0 * u,
                    y + 5.0 * u,
                    28.0 * u,
                    18.0 * u,
                    self.pal.error,
                    r4(9.0 * u),
                );
                text_c(
                    ui,
                    "99+",
                    x + 156.0 * u,
                    y + 6.0 * u,
                    ts(M3_LABEL_SMALL, self.pal.on_error, u),
                );

                ui.text(
                    "Small (8dp), Count, and Pill (99+) Badges",
                    x + 184.0 * u,
                    y + 18.0 * u,
                    ts(M3_BODY_MEDIUM, self.pal.sub, u),
                );
            }
            ComponentId::ListItem => {
                // M3 Expressive Segmented List: 16dp outer corners, 4dp inner corners, 2dp gap, M3Shape avatars
                let rows = [
                    (
                        MaterialShape::Clover4Leaf,
                        "person",
                        "Single-line Expressive item",
                        "",
                        "12:45",
                        56.0f32,
                    ),
                    (
                        MaterialShape::Sunny,
                        "star",
                        "Two-line Expressive item",
                        "Supporting text with crisp M3E container segmentation.",
                        "Yesterday",
                        72.0,
                    ),
                    (
                        MaterialShape::Cookie9Sided,
                        "description",
                        "Three-line Expressive item",
                        "First line of supporting copy for rich list items.",
                        "Oct 14",
                        88.0,
                    ),
                ];
                let mut ry = y;
                for (i, (sh, ic, head, support, tr, h)) in rows.into_iter().enumerate() {
                    let hh = h * u;
                    let radii = match i {
                        0 => [16.0 * u, 16.0 * u, 4.0 * u, 4.0 * u],
                        1 => r4(4.0 * u),
                        _ => [4.0 * u, 4.0 * u, 16.0 * u, 16.0 * u],
                    };
                    let rect = Rect { x, y: ry, w, h: hh };
                    ui.rect(x, ry, w, hh, self.pal.card_hi, radii);
                    let a = Action::Button(110 + i as u32);
                    self.layer_radii(ui, a, rect, radii, self.pal.text);
                    self.draw_ripples(ui, rect);
                    ui.m3_shape(
                        x + 28.0 * u,
                        ry + hh * 0.5,
                        36.0 * u,
                        sh,
                        0.0,
                        self.pal.primary_container,
                        [0.0; 4],
                        0.0,
                    );
                    icon_c(
                        ui,
                        ic,
                        x + 28.0 * u,
                        ry + hh * 0.5,
                        18.0 * u,
                        self.pal.on_primary_container,
                    );
                    ui.text(
                        head,
                        x + 56.0 * u,
                        ry + if h >= 88.0 { 12.0 } else { 16.0 } * u,
                        ts(M3_BODY_LARGE, self.pal.text, u),
                    );
                    if !support.is_empty() {
                        ui.text(
                            support,
                            x + 56.0 * u,
                            ry + if h >= 88.0 { 36.0 } else { 40.0 } * u,
                            ts(M3_BODY_MEDIUM, self.pal.sub, u),
                        );
                    }
                    if h >= 88.0 {
                        ui.text(
                            "Second line of supporting description copy.",
                            x + 56.0 * u,
                            ry + 56.0 * u,
                            ts(M3_BODY_MEDIUM, self.pal.sub, u),
                        );
                    }
                    let style = ts(M3_LABEL_SMALL, self.pal.sub, u);
                    let tw = ui.text_width(tr, style.px, style.medium, style.tracking_em);
                    ui.text(tr, x + w - 16.0 * u - tw, ry + 18.0 * u, style);
                    self.zone(a, rect, self.pal.text);
                    ry += hh + 2.0 * u;
                }
            }
            ComponentId::Tabs => {
                // Row 1: PrimaryTabRow (icon + label + 3dp top-rounded active indicator)
                let items = [
                    ("code", "Code"),
                    ("terminal", "Build"),
                    ("send", "Deploy"),
                    ("description", "Docs"),
                ];
                let tw2 = w / 4.0;
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let tx = x + i as f32 * tw2;
                    let sel = self.tabs == i;
                    let col = if sel { self.pal.primary } else { self.pal.sub };
                    icon_c(ui, ic, tx + tw2 * 0.5, y + 18.0 * u, 20.0 * u, col);
                    text_c(
                        ui,
                        lb,
                        tx + tw2 * 0.5,
                        y + 34.0 * u,
                        ts(M3_TITLE_SMALL, col, u),
                    );
                    if sel {
                        ui.rect(
                            tx + tw2 * 0.5 - 22.0 * u,
                            y + 57.0 * u,
                            44.0 * u,
                            3.0 * u,
                            self.pal.primary,
                            [3.0 * u, 3.0 * u, 0.0, 0.0],
                        );
                    }
                    let rect = Rect {
                        x: tx,
                        y,
                        w: tw2,
                        h: 60.0 * u,
                    };
                    self.layer(ui, Action::Tab(i), rect, 8.0 * u, self.pal.text);
                    self.draw_ripples(ui, rect);
                    self.zone(Action::Tab(i), rect, self.pal.text);
                }
                ui.rect(
                    x,
                    y + 60.0 * u,
                    w,
                    1.0 * u,
                    self.pal.outline_variant,
                    r4(0.0),
                );

                // Row 2: SecondaryTabRow (text-only with full-width indicator)
                let sec_items = ["Overview", "Specifications", "Guidelines"];
                let sw = w / 3.0;
                let sy = y + 70.0 * u;
                for (i, lb) in sec_items.into_iter().enumerate() {
                    let tx = x + i as f32 * sw;
                    let sel = self.sec_tabs == i;
                    let col = if sel { self.pal.text } else { self.pal.sub };
                    text_c(
                        ui,
                        lb,
                        tx + sw * 0.5,
                        sy + 12.0 * u,
                        ts(M3_TITLE_SMALL, col, u),
                    );
                    if sel {
                        ui.rect(
                            tx,
                            sy + 42.0 * u,
                            sw,
                            2.0 * u,
                            self.pal.primary,
                            r4(1.0 * u),
                        );
                    }
                    let rect = Rect {
                        x: tx,
                        y: sy,
                        w: sw,
                        h: 44.0 * u,
                    };
                    self.layer(ui, Action::SecondaryTab(i), rect, 8.0 * u, self.pal.text);
                    self.draw_ripples(ui, rect);
                    self.zone(Action::SecondaryTab(i), rect, self.pal.text);
                }
                ui.rect(
                    x,
                    sy + 44.0 * u,
                    w,
                    1.0 * u,
                    self.pal.outline_variant,
                    r4(0.0),
                );
            }
            ComponentId::SegmentedButton => {
                let labels = ["Expressive", "Standard", "Custom"];
                let sw = w / 3.0;
                let h = 40.0 * u;
                for (i, lb) in labels.into_iter().enumerate() {
                    let sx = x + i as f32 * sw;
                    let sel = self.segmented == i;
                    let end = 20.0 * u;
                    let seg_r = if i == 0 {
                        [end, 0.0, 0.0, end]
                    } else if i == labels.len() - 1 {
                        [0.0, end, end, 0.0]
                    } else {
                        r4(0.0)
                    };
                    let fill = if sel {
                        self.pal.secondary_container
                    } else {
                        [0.0; 4]
                    };
                    ui.rect_styled(
                        sx,
                        y + 4.0 * u,
                        sw,
                        h,
                        fill,
                        self.pal.outline,
                        1.0 * u,
                        seg_r,
                        0.0,
                    );
                    let ink = if sel {
                        self.pal.on_secondary_container
                    } else {
                        self.pal.text
                    };
                    let style = ts(M3_LABEL_LARGE, ink, u);
                    let ltw = ui.text_width(lb, style.px, style.medium, style.tracking_em);
                    let icon_w = if sel { 24.0 * u } else { 0.0 };
                    let start = sx + (sw - ltw - icon_w) * 0.5;
                    if sel {
                        icon_c(ui, "check", start + 8.0 * u, y + 24.0 * u, 18.0 * u, ink);
                    }
                    ui.text(lb, start + icon_w, y + 24.0 * u - style.line_h * 0.5, style);
                    let rect = Rect {
                        x: sx,
                        y: y + 4.0 * u,
                        w: sw,
                        h,
                    };
                    self.layer_radii(ui, Action::Segmented(i), rect, seg_r, self.pal.text);
                    self.draw_ripples(ui, rect);
                    self.zone(Action::Segmented(i), rect, self.pal.text);
                }
            }
            ComponentId::Divider => {
                // Full-width divider + Inset divider + Subheader divider
                ui.text(
                    "Full-bleed & Inset Dividers",
                    x,
                    y,
                    ts(M3_LABEL_SMALL, self.pal.sub, u),
                );
                ui.rect(
                    x,
                    y + 22.0 * u,
                    w,
                    1.0 * u,
                    self.pal.outline_variant,
                    r4(0.5 * u),
                );
                ui.rect(
                    x + 56.0 * u,
                    y + 40.0 * u,
                    (w - 72.0 * u).max(40.0 * u),
                    1.0 * u,
                    self.pal.outline_variant,
                    r4(0.5 * u),
                );
            }
            ComponentId::PullToRefresh => {
                // Authentic M3 Expressive PullToRefreshBox: elevated circular container with morphing LoadingIndicator!
                let cx = x + 32.0 * u;
                let cy = y + 36.0 * u;
                ui.circle(cx, cy + 2.0 * u, 24.0 * u, with_alpha(self.pal.bg, 0.65));
                ui.circle(cx, cy, 24.0 * u, self.pal.card_highest);
                let (sa, sb, mt, rot, sc) = loading_morph_state(self.t, 1);
                ui.m3_shape_morph(
                    cx,
                    cy,
                    32.0 * u * sc,
                    sa,
                    sb,
                    mt,
                    rot,
                    self.pal.primary,
                    [0.0; 4],
                    0.0,
                );
                ui.text(
                    "Refreshing feed...",
                    x + 72.0 * u,
                    y + 14.0 * u,
                    ts(M3_TITLE_SMALL, self.pal.text, u),
                );
                ui.text(
                    "PullToRefreshBox uses ContainedLoadingIndicator",
                    x + 72.0 * u,
                    y + 36.0 * u,
                    ts(M3_BODY_SMALL, self.pal.sub, u),
                );
            }
            ComponentId::SwipeToDismiss => {
                ui.rect(
                    x,
                    y + 4.0 * u,
                    w,
                    56.0 * u,
                    self.pal.error_container,
                    r4(16.0 * u),
                );
                icon_c(
                    ui,
                    "delete",
                    x + w - 28.0 * u,
                    y + 32.0 * u,
                    22.0 * u,
                    self.pal.on_error_container,
                );
                ui.text(
                    "Delete",
                    x + w - 92.0 * u,
                    y + 22.0 * u,
                    ts(M3_LABEL_LARGE, self.pal.on_error_container, u),
                );
                let fw = w - 108.0 * u;
                ui.rect(x, y + 4.0 * u, fw, 56.0 * u, self.pal.card_hi, r4(16.0 * u));
                ui.m3_shape(
                    x + 28.0 * u,
                    y + 32.0 * u,
                    32.0 * u,
                    MaterialShape::Arch,
                    0.0,
                    self.pal.primary_container,
                    [0.0; 4],
                    0.0,
                );
                icon_c(
                    ui,
                    "mail",
                    x + 28.0 * u,
                    y + 32.0 * u,
                    16.0 * u,
                    self.pal.on_primary_container,
                );
                ui.text(
                    "Swipe to dismiss item",
                    x + 54.0 * u,
                    y + 20.0 * u,
                    ts(M3_BODY_LARGE, self.pal.text, u),
                );
            }
            ComponentId::Toolbar => {
                // M3 Expressive HorizontalFloatingToolbar pill + adjacent 56dp FAB!
                let icons = ["edit", "palette", "brush", "share", "more_vert"];
                let tw3 = icons.len() as f32 * 44.0 * u + 16.0 * u;
                let th = 56.0 * u;
                ui.rect(
                    x,
                    y + 4.0 * u,
                    tw3,
                    th,
                    with_alpha(self.pal.bg, 0.7),
                    r4(28.0 * u),
                );
                ui.rect(
                    x,
                    y + 2.0 * u,
                    tw3,
                    th,
                    self.pal.primary_container,
                    r4(28.0 * u),
                );
                for (i, ic) in icons.into_iter().enumerate() {
                    let bx = x + 8.0 * u + i as f32 * 44.0 * u;
                    let by = y + 10.0 * u;
                    let id = 40 + i as u32;
                    let rect = Rect {
                        x: bx,
                        y: by,
                        w: 40.0 * u,
                        h: 40.0 * u,
                    };
                    if i == 0 {
                        ui.rect(bx, by, 40.0 * u, 40.0 * u, self.pal.primary, r4(20.0 * u));
                    }
                    self.layer(
                        ui,
                        Action::Button(id),
                        rect,
                        20.0 * u,
                        self.pal.on_primary_container,
                    );
                    self.draw_ripples(ui, rect);
                    icon_c(
                        ui,
                        ic,
                        bx + 20.0 * u,
                        by + 20.0 * u,
                        22.0 * u,
                        if i == 0 {
                            self.pal.on_primary
                        } else {
                            self.pal.on_primary_container
                        },
                    );
                    self.zone(Action::Button(id), rect, self.pal.on_primary_container);
                }
                // Adjacent 56dp Secondary FAB per FloatingToolbarWithFab spec
                self.fab_styled(
                    ui,
                    Action::Button(49),
                    x + tw3 + 12.0 * u,
                    y + 2.0 * u,
                    56.0 * u,
                    16.0 * u,
                    self.pal.secondary_container,
                    self.pal.on_secondary_container,
                    "add",
                    24.0 * u,
                );
            }
        }
    }
}

pub struct GalleryApp {
    gallery: Gallery,
    ui: Option<UiInner>,
    last_frame: Option<Instant>,
}

struct UiInner {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    renderer: UiRenderer,
}

impl GalleryApp {
    pub fn new() -> Self {
        Self {
            gallery: Gallery::new(),
            ui: None,
            last_frame: None,
        }
    }

    pub fn resume(&mut self, window: Arc<Window>) -> bool {
        use log::{error, info};
        use wgpu::{
            CompositeAlphaMode, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
            InstanceDescriptor, MemoryHints, PowerPreference, PresentMode, RequestAdapterOptions,
            TextureFormat, TextureUsages, Trace,
        };
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let surface = match instance.create_surface(window.clone()) {
            Ok(surface) => surface,
            Err(err) => {
                error!("surface creation failed: {err:?}");
                return false;
            }
        };
        let adapter = match pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
            apply_limit_buckets: false,
        })) {
            Ok(adapter) => adapter,
            Err(err) => {
                error!("adapter request failed: {err:?}");
                return false;
            }
        };
        let (device, queue) = match pollster::block_on(adapter.request_device(&DeviceDescriptor {
            label: None,
            required_features: Features::empty(),
            required_limits: adapter.limits(),
            experimental_features: ExperimentalFeatures::disabled(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
        })) {
            Ok(pair) => pair,
            Err(err) => {
                error!("device request failed: {err:?}");
                return false;
            }
        };
        let caps = surface.get_capabilities(&adapter);
        let size = window.inner_size();
        let format = caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(TextureFormat::Bgra8UnormSrgb);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(CompositeAlphaMode::Auto),
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let mut renderer = UiRenderer::new(&device, &queue, config.format);
        renderer.set_screen(&device, &queue, config.width as f32, config.height as f32);
        info!("gallery renderer ready {}x{}", config.width, config.height);
        self.last_frame = None;
        self.ui = Some(UiInner {
            window,
            surface,
            device,
            queue,
            config,
            renderer,
        });
        true
    }

    pub fn suspend(&mut self) {
        self.ui = None;
        self.last_frame = None;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(ui) = self.ui.as_mut() {
            ui.config.width = width.max(1);
            ui.config.height = height.max(1);
            ui.surface.configure(&ui.device, &ui.config);
            ui.renderer.set_screen(
                &ui.device,
                &ui.queue,
                ui.config.width as f32,
                ui.config.height as f32,
            );
        }
    }

    pub fn frame(&mut self) {
        use wgpu::CurrentSurfaceTexture;
        let Some(ui) = self.ui.as_mut() else {
            return;
        };
        let now = Instant::now();
        let dt = self
            .last_frame
            .map_or(1.0 / 60.0, |prev| (now - prev).as_secs_f32());
        self.last_frame = Some(now);
        self.gallery.step(dt);
        let size = ui.window.inner_size();
        if size.width != ui.config.width || size.height != ui.config.height {
            ui.config.width = size.width.max(1);
            ui.config.height = size.height.max(1);
            ui.surface.configure(&ui.device, &ui.config);
            ui.renderer.set_screen(
                &ui.device,
                &ui.queue,
                ui.config.width as f32,
                ui.config.height as f32,
            );
        }
        let frame = match ui.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) | CurrentSurfaceTexture::Suboptimal(frame) => {
                frame
            }
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                ui.config.width = size.width.max(1);
                ui.config.height = size.height.max(1);
                ui.surface.configure(&ui.device, &ui.config);
                ui.renderer.set_screen(
                    &ui.device,
                    &ui.queue,
                    ui.config.width as f32,
                    ui.config.height as f32,
                );
                ui.window.request_redraw();
                return;
            }
            _ => {
                ui.window.request_redraw();
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.gallery.draw(
            &mut ui.renderer,
            ui.config.width as f32,
            ui.config.height as f32,
        );
        ui.renderer
            .flush(&ui.device, &ui.queue, &view, self.gallery.bg());
        ui.queue.present(frame);
        ui.window.request_redraw();
    }

    pub fn scroll(&mut self, dy: f32) {
        self.gallery.scroll_by(dy);
    }

    pub fn pointer_down(&mut self, x: f32, y: f32) {
        self.gallery.down(x, y);
    }

    pub fn pointer_move(&mut self, x: f32, y: f32) {
        self.gallery.move_to(x, y);
    }

    pub fn pointer_up(&mut self, x: f32, y: f32) {
        self.gallery.up(x, y);
    }
}
