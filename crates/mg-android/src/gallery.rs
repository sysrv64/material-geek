use mg_components::ComponentId;
use mg_motion::{cubic_bezier_y, DynamicSpring, MotionScheme, Tempo, Track};
use mg_render::{
    m3_text_style, M3_BODY_LARGE, M3_BODY_MEDIUM, M3_BODY_SMALL, M3_DISPLAY_SMALL,
    M3_HEADLINE_SMALL, M3_LABEL_LARGE, M3_LABEL_MEDIUM, M3_LABEL_SMALL, M3_TITLE_LARGE,
    M3_TITLE_MEDIUM, M3_TITLE_SMALL, TypeStyle, TextStyle, UiRenderer,
};
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
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, 1.0]
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

#[derive(Clone, Copy, PartialEq)]
enum Action {
    Switch(usize),
    Check(usize),
    Radio(usize),
    Chip(usize),
    Tab(usize),
    Nav(usize),
    Rail(usize),
    Drawer(usize),
    Group(usize),
    Segmented(usize),
    MenuItem(usize),
    CarouselPage(usize),
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
    card: [f32; 4],
    card_hi: [f32; 4],
    text: [f32; 4],
    sub: [f32; 4],
    primary: [f32; 4],
    on_primary: [f32; 4],
    primary_container: [f32; 4],
    on_primary_container: [f32; 4],
    secondary_container: [f32; 4],
    on_secondary_container: [f32; 4],
    tertiary_container: [f32; 4],
    error: [f32; 4],
    on_error: [f32; 4],
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
            card: argb(0x2B2930),
            card_hi: argb(0x36343B),
            text: argb(0xE6E0E9),
            sub: argb(0xCAC4D0),
            primary: argb(0xD0BCFF),
            on_primary: argb(0x381E72),
            primary_container: argb(0x4F378B),
            on_primary_container: argb(0xEADDFF),
            secondary_container: argb(0x4A4458),
            on_secondary_container: argb(0xE8DEF8),
            tertiary_container: argb(0x633B48),
            error: argb(0xF2B8B5),
            on_error: argb(0x601410),
            outline: argb(0x938F99),
            outline_variant: argb(0x49454F),
            inverse: argb(0xE6E0E9),
            on_inverse: argb(0x313033),
            inverse_primary: argb(0xD0BCFF),
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
    slider_thumb: DynamicSpring,
    slider: f32,
    focus: DynamicSpring,
    switches: [bool; 2],
    switch_anim: [DynamicSpring; 2],
    check_state: [u8; 3],
    check_draw: [DynamicSpring; 3],
    check_color: [DynamicSpring; 3],
    check_uncheck_at: [f32; 3],
    radio: usize,
    radio_anim: [DynamicSpring; 3],
    chips: [bool; 4],
    chip_anim: [DynamicSpring; 4],
    tabs: usize,
    nav: usize,
    rail: usize,
    drawer: usize,
    segmented: usize,
    menu_sel: usize,
    carousel: usize,
    fab_open: bool,
    zones: Vec<(Action, Rect, [f32; 4])>,
}

impl Gallery {
    pub fn new() -> Self {
        let scheme = MotionScheme::expressive();
        let fs = scheme.spec(Tempo::Fast, Track::Spatial);
        let ds = scheme.spec(Tempo::Default, Track::Spatial);
        let dfx = scheme.spec(Tempo::Default, Track::Effects);
        let switches = [true, false];
        let check_state = [0u8, 1, 2];
        let radio = 0usize;
        let chips = [false, true, false, false];
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
            morph: DynamicSpring::new(0.0, 0.0, dfx),
            morph_key: 0,
            group_press: DynamicSpring::new(0.0, 0.0, fs),
            slider_thumb: DynamicSpring::new(0.0, 0.0, fs),
            slider: 0.6,
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
            tabs: 0,
            nav: 0,
            rail: 0,
            drawer: 0,
            segmented: 0,
            menu_sel: 1,
            carousel: 0,
            fab_open: false,
            zones: Vec::new(),
        }
    }

    pub fn bg(&self) -> [f32; 4] {
        self.pal.bg
    }

    fn unit(width: f32) -> f32 {
        (width / 480.0).max(1.0)
    }

    pub fn step(&mut self) {
        let dt = 1.0 / 60.0;
        self.morph.step(dt);
        self.group_press.step(dt);
        self.slider_thumb.step(dt);
        self.focus.step(dt);
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
        let bounded = !matches!(action, Action::Switch(_) | Action::Check(_) | Action::Radio(_));
        self.spawn_ripple(rect, x, y, bounded, ink);
        self.pressed = Some(Pressed { action, start: self.t });
        if let Action::Button(id) = action {
            self.morph_key = id;
            self.morph.retarget(1.0);
        }
        if matches!(action, Action::Group(_)) {
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
            Action::Tab(i) => self.tabs = i,
            Action::Nav(i) => self.nav = i,
            Action::Rail(i) => self.rail = i,
            Action::Drawer(i) => self.drawer = i,
            Action::Group(i) | Action::Segmented(i) => self.segmented = i,
            Action::MenuItem(i) => self.menu_sel = i,
            Action::CarouselPage(i) => self.carousel = i,
            Action::FabMenu => self.fab_open = !self.fab_open,
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
            let grow =
                cubic_bezier_y(((now - r.start) / 0.225).clamp(0.0, 1.0), 0.2, 0.0, 0.0, 1.0);
            let end_r = if r.bounded {
                (rect.w * rect.w + rect.h * rect.h).sqrt() * 0.5 + 10.0 * u
            } else {
                20.0 * u
            };
            let start_r = if r.bounded { 0.3 * rect.w.max(rect.h) } else { 0.0 };
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
            ui.rect(rect.x, rect.y, rect.w, rect.h, with_alpha(ink, 0.10), r4(radius));
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
            let f = cubic_bezier_y(((self.t - p.start) / 0.12).clamp(0.0, 1.0), 0.2, 0.0, 0.0, 1.0);
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

    fn arc(&self, ui: &mut UiRenderer, cx: f32, cy: f32, r: f32, start_deg: f32, sweep_deg: f32) {
        let n = 36;
        let dot = 2.0 * self.u;
        let color = self.pal.primary;
        for k in 0..n {
            let frac = if sweep_deg >= 359.0 {
                k as f32 / n as f32
            } else {
                k as f32 / (n - 1) as f32
            };
            let ang = (start_deg + sweep_deg * frac).to_radians();
            ui.circle(cx + r * ang.cos(), cy + r * ang.sin(), dot, color);
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
            let name = format!("{id:?}");
            ui.text(&name, pad, y, ts(M3_LABEL_MEDIUM, self.pal.sub, u));
            let body_h = Self::body_height(id, u);
            let card_y = y + 24.0 * u;
            ui.rect(card_x, card_y, card_w, body_h + 32.0 * u, self.pal.card, r4(12.0 * u));
            self.draw_body(ui, id, card_x + 16.0 * u, card_y + 16.0 * u, card_w - 32.0 * u);
            y = card_y + body_h + 32.0 * u + 20.0 * u;
        }
        self.content_h = y + self.scroll + 16.0 * u;
        self.scroll_by(0.0);
        self.draw_app_bar(ui, width);
    }

    fn draw_app_bar(&mut self, ui: &mut UiRenderer, width: f32) {
        let u = self.u;
        ui.rect(0.0, 0.0, width, 64.0 * u, self.pal.bg, r4(0.0));
        self.icon_button(ui, 100, 4.0 * u, 12.0 * u, "menu", IconBtn::Standard);
        ui.text(
            "Component gallery",
            56.0 * u,
            18.0 * u,
            ts(M3_TITLE_LARGE, self.pal.text, u),
        );
        self.icon_button(ui, 101, width - 88.0 * u, 12.0 * u, "search", IconBtn::Standard);
        self.icon_button(ui, 102, width - 44.0 * u, 12.0 * u, "more_vert", IconBtn::Standard);
        ui.rect(0.0, 63.0 * u, width, 1.0 * u, self.pal.outline_variant, r4(0.5 * u));
    }

    fn body_height(id: ComponentId, u: f32) -> f32 {
        match id {
            ComponentId::Button => 92.0 * u,
            ComponentId::IconButton => 48.0 * u,
            ComponentId::Fab => 56.0 * u,
            ComponentId::ExtendedFab => 56.0 * u,
            ComponentId::FabMenu => 220.0 * u,
            ComponentId::SplitButton => 48.0 * u,
            ComponentId::ButtonGroup => 48.0 * u,
            ComponentId::Card => 140.0 * u,
            ComponentId::Checkbox => 120.0 * u,
            ComponentId::Switch => 96.0 * u,
            ComponentId::RadioButton => 120.0 * u,
            ComponentId::Slider => 84.0 * u,
            ComponentId::ProgressIndicator => 132.0 * u,
            ComponentId::LoadingIndicator => 64.0 * u,
            ComponentId::AssistChip
            | ComponentId::FilterChip
            | ComponentId::InputChip
            | ComponentId::SuggestionChip => 44.0 * u,
            ComponentId::Dialog => 224.0 * u,
            ComponentId::BottomSheet => 184.0 * u,
            ComponentId::NavigationBar => 80.0 * u,
            ComponentId::NavigationRail => 232.0 * u,
            ComponentId::NavigationDrawer => 288.0 * u,
            ComponentId::Scaffold => 232.0 * u,
            ComponentId::TopAppBar => 72.0 * u,
            ComponentId::SearchBar => 64.0 * u,
            ComponentId::TextField => 148.0 * u,
            ComponentId::Menu => 204.0 * u,
            ComponentId::Carousel => 176.0 * u,
            ComponentId::DatePicker => 288.0 * u,
            ComponentId::TimePicker => 200.0 * u,
            ComponentId::Tooltip => 112.0 * u,
            ComponentId::Snackbar => 64.0 * u,
            ComponentId::Badge => 56.0 * u,
            ComponentId::ListItem => 216.0 * u,
            ComponentId::Tabs => 68.0 * u,
            ComponentId::SegmentedButton => 48.0 * u,
            ComponentId::Divider => 24.0 * u,
            ComponentId::PullToRefresh => 68.0 * u,
            ComponentId::SwipeToDismiss => 64.0 * u,
            ComponentId::Toolbar => 56.0 * u,
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
            24.0 * u
        };
        let icon_w = if icon.is_some() { 26.0 * u } else { 0.0 };
        let mut w = pad_h + icon_w + tw + pad_h;
        if !matches!(variant, Btn::Text) && w < 58.0 * u {
            w = 58.0 * u;
        }
        let radius = if self.morph_key == id {
            lerp(h * 0.5, 8.0 * u, self.morph.x)
        } else {
            h * 0.5
        };
        if variant == Btn::Elevated {
            ui.rect(x, y + 2.0 * u, w, h, with_alpha(self.pal.bg, 0.6), r4(radius));
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
        ui.text(label, x + pad_h + icon_w, y + h * 0.5 - style.line_h * 0.5, style);
        self.zone(Action::Button(id), Rect { x, y, w, h }, ink);
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
        let u = self.u;
        let s = 40.0 * u;
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
        let radius = if self.morph_key == id {
            lerp(20.0 * u, 8.0 * u, self.morph.x)
        } else {
            20.0 * u
        };
        if let Some(f) = fill {
            ui.rect(x, y, s, s, f, r4(radius));
        }
        if border {
            ui.stroke(x, y, s, s, self.pal.outline, r4(radius), 1.0 * u);
        }
        self.layer(ui, Action::Button(id), Rect { x, y, w: s, h: s }, radius, ink);
        self.draw_ripples(ui, Rect { x, y, w: s, h: s });
        icon_c(ui, icon, x + s * 0.5, y + s * 0.5, 24.0 * u, ink);
        self.zone(Action::Button(id), Rect { x, y, w: s, h: s }, ink);
    }

    fn fab(
        &mut self,
        ui: &mut UiRenderer,
        action: Action,
        x: f32,
        y: f32,
        size: f32,
        icon: &str,
    ) -> f32 {
        let u = self.u;
        let radius = if size >= 56.0 * u { 16.0 * u } else { 12.0 * u };
        let e = self.elev_dp(action) * u;
        let ink = self.pal.on_primary_container;
        ui.rect(x, y + e, size, size, with_alpha(self.pal.bg, 0.7), r4(radius));
        ui.rect(x, y, size, size, self.pal.primary_container, r4(radius));
        self.layer(ui, action, Rect { x, y, w: size, h: size }, radius, ink);
        self.draw_ripples(ui, Rect { x, y, w: size, h: size });
        icon_c(ui, icon, x + size * 0.5, y + size * 0.5, 24.0 * u, ink);
        self.zone(action, Rect { x, y, w: size, h: size }, ink);
        size
    }

    fn group_segment(&mut self, ui: &mut UiRenderer, i: usize, x: f32, y: f32, label: &str) {
        let u = self.u;
        let w = 104.0 * u;
        let h = 40.0 * u;
        let expand = if self.is_pressed(Action::Group(i)) {
            self.group_press.x
        } else {
            0.0
        };
        let sel = self.segmented == i;
        let fill = if sel {
            self.pal.secondary_container
        } else {
            self.pal.card_hi
        };
        let ink = if sel {
            self.pal.on_secondary_container
        } else {
            self.pal.text
        };
        let ex = expand * 8.0 * u;
        let radius = expand * 20.0 * u;
        let rect = Rect {
            x: x - ex * 0.5,
            y,
            w: w + ex,
            h,
        };
        ui.rect(rect.x, rect.y, rect.w, rect.h, fill, r4(radius));
        self.layer(ui, Action::Group(i), rect, radius, self.pal.text);
        self.draw_ripples(ui, rect);
        let style = ts(M3_LABEL_LARGE, ink, u);
        let tw = ui.text_width(label, style.px, style.medium, style.tracking_em);
        ui.text(label, x + (w - tw) * 0.5, y + h * 0.5 - style.line_h * 0.5, style);
        self.zone(Action::Group(i), Rect { x, y, w, h }, self.pal.text);
    }

    fn draw_checkbox(&mut self, ui: &mut UiRenderer, i: usize, x: f32, ry: f32, label: &str) {
        let u = self.u;
        let st = self.check_state[i];
        let ca = if st == 0 { 0.0 } else { self.check_color[i].x };
        let cf = self.check_draw[i].x;
        let bx = x + 11.0 * u;
        let by = ry + 11.0 * u;
        let hit = Rect { x, y: ry, w: 40.0 * u, h: 40.0 * u };
        self.layer_circle(ui, Action::Check(i), x + 20.0 * u, ry + 20.0 * u, 20.0 * u, self.pal.text);
        self.draw_ripples(ui, hit);
        if ca > 0.0 {
            ui.rect(bx, by, 18.0 * u, 18.0 * u, with_alpha(self.pal.primary, ca), r4(2.0 * u));
        }
        if ca < 1.0 {
            ui.stroke(
                bx,
                by,
                18.0 * u,
                18.0 * u,
                with_alpha(self.pal.outline, 1.0 - ca),
                r4(2.0 * u),
                2.0 * u,
            );
        }
        if st == 2 {
            ui.rect(bx + 4.0 * u, by + 8.0 * u, 10.0 * u, 2.0 * u, with_alpha(self.pal.on_primary, ca), r4(1.0 * u));
        } else if cf > 0.02 {
            icon_c(
                ui,
                "check",
                bx + 9.0 * u,
                by + 9.0 * u,
                18.0 * u * cf,
                self.pal.on_primary,
            );
            ui.clip_last([bx, by, 18.0 * u, 18.0 * u]);
        }
        ui.text(label, x + 48.0 * u, ry + 8.0 * u, ts(M3_BODY_LARGE, self.pal.text, u));
        self.zone(
            Action::Check(i),
            Rect { x, y: ry, w: 220.0 * u, h: 40.0 * u },
            self.pal.text,
        );
    }

    fn draw_switch(&mut self, ui: &mut UiRenderer, i: usize, x: f32, ry: f32, label: &str) {
        let u = self.u;
        let on = self.switches[i];
        let anim = self.switch_anim[i].x;
        let pressed = self.is_pressed(Action::Switch(i));
        let ty = ry + 6.0 * u;
        let cy = ry + 22.0 * u;
        if on {
            ui.rect(x, ty, 52.0 * u, 32.0 * u, self.pal.primary, r4(16.0 * u));
        } else {
            ui.rect(x, ty, 52.0 * u, 32.0 * u, self.pal.card_hi, r4(16.0 * u));
            ui.stroke(x, ty, 52.0 * u, 32.0 * u, self.pal.outline, r4(16.0 * u), 2.0 * u);
        }
        self.layer_circle(ui, Action::Switch(i), x + 26.0 * u, cy, 20.0 * u, self.pal.text);
        self.draw_ripples(ui, Rect { x, y: ry, w: 52.0 * u, h: 44.0 * u });
        let size = if pressed {
            28.0 * u
        } else {
            lerp(16.0 * u, 24.0 * u, anim)
        };
        let cx = if pressed {
            if on {
                32.0 * u
            } else {
                16.0 * u
            }
        } else {
            lerp(14.0 * u, 34.0 * u, anim)
        };
        let thumb = mix(self.pal.outline, self.pal.on_primary, anim);
        ui.circle(x + cx, cy, size * 0.5, thumb);
        ui.text(label, x + 68.0 * u, ry + 10.0 * u, ts(M3_BODY_LARGE, self.pal.text, u));
        self.zone(
            Action::Switch(i),
            Rect { x, y: ry, w: 120.0 * u, h: 44.0 * u },
            self.pal.text,
        );
    }

    fn draw_radio(&mut self, ui: &mut UiRenderer, i: usize, x: f32, ry: f32, label: &str) {
        let u = self.u;
        let sel = self.radio == i;
        let cx = x + 20.0 * u;
        let cy = ry + 20.0 * u;
        let ring = if sel { self.pal.primary } else { self.pal.outline };
        self.layer_circle(ui, Action::Radio(i), cx, cy, 20.0 * u, self.pal.text);
        self.draw_ripples(ui, Rect { x, y: ry, w: 40.0 * u, h: 40.0 * u });
        ui.circle(cx, cy, 10.0 * u, ring);
        ui.circle(cx, cy, 8.0 * u, self.pal.card);
        let dot = self.radio_anim[i].x;
        if dot > 0.02 {
            ui.circle(cx, cy, 6.0 * u * dot, self.pal.primary);
        }
        ui.text(label, x + 48.0 * u, ry + 8.0 * u, ts(M3_BODY_LARGE, self.pal.text, u));
        self.zone(
            Action::Radio(i),
            Rect { x, y: ry, w: 220.0 * u, h: 40.0 * u },
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
        let radius = lerp(8.0 * u, 16.0 * u, self.chip_anim[i].x);
        let ink = if sel {
            self.pal.on_secondary_container
        } else {
            self.pal.text
        };
        let style = ts(M3_LABEL_LARGE, ink, u);
        let tw = ui.text_width(label, style.px, style.medium, style.tracking_em);
        let lead_w = if leading.is_some() { 26.0 * u } else { 0.0 };
        let trail_w = if trailing.is_some() { 26.0 * u } else { 0.0 };
        let w = 8.0 * u + lead_w + tw + trail_w + 8.0 * u;
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
                x + 8.0 * u + 9.0 * u,
                y + h * 0.5,
                18.0 * u,
                if sel {
                    self.pal.on_secondary_container
                } else {
                    self.pal.sub
                },
            );
        }
        let tx = x + 8.0 * u + lead_w;
        ui.text(label, tx, y + h * 0.5 - style.line_h * 0.5, style);
        if let Some(ic) = trailing {
            icon_c(ui, ic, tx + tw + 8.0 * u + 9.0 * u, y + h * 0.5, 18.0 * u, self.pal.sub);
        }
        self.zone(Action::Chip(i), rect, self.pal.text);
        w
    }

    fn draw_body(&mut self, ui: &mut UiRenderer, id: ComponentId, x: f32, y: f32, w: f32) {
        let u = self.u;
        match id {
            ComponentId::Button => {
                let mut bx = x;
                bx += self.draw_button(ui, 1, bx, y, BtnSpec { label: "Filled", icon: None, variant: Btn::Filled })
                    + 12.0 * u;
                bx += self.draw_button(ui, 2, bx, y, BtnSpec { label: "Tonal", icon: None, variant: Btn::Tonal })
                    + 12.0 * u;
                self.draw_button(ui, 3, bx, y, BtnSpec { label: "Outlined", icon: None, variant: Btn::Outlined });
                let mut bx = x;
                bx += self.draw_button(
                    ui,
                    4,
                    bx,
                    y + 52.0 * u,
                    BtnSpec { label: "Elevated", icon: None, variant: Btn::Elevated },
                ) + 12.0 * u;
                bx += self.draw_button(ui, 5, bx, y + 52.0 * u, BtnSpec { label: "Text", icon: None, variant: Btn::Text })
                    + 12.0 * u;
                self.draw_button(
                    ui,
                    6,
                    bx,
                    y + 52.0 * u,
                    BtnSpec { label: "Favorite", icon: Some("favorite"), variant: Btn::Filled },
                );
            }
            ComponentId::IconButton => {
                let variants = [
                    IconBtn::Standard,
                    IconBtn::Filled,
                    IconBtn::Tonal,
                    IconBtn::Outlined,
                ];
                let icons = ["favorite", "add", "settings", "close"];
                for (i, v) in variants.into_iter().enumerate() {
                    self.icon_button(ui, 10 + i as u32, x + i as f32 * 52.0 * u, y + 4.0 * u, icons[i], v);
                }
            }
            ComponentId::Fab => {
                let mut fx = x;
                fx += self.fab(ui, Action::Button(20), fx, y, 56.0 * u, "add") + 16.0 * u;
                self.fab(ui, Action::Button(24), fx, y + 8.0 * u, 40.0 * u, "edit");
            }
            ComponentId::ExtendedFab => {
                let ink = self.pal.on_primary_container;
                let style = ts(M3_LABEL_LARGE, ink, u);
                let tw = ui.text_width("Compose", style.px, style.medium, style.tracking_em);
                let fw = 16.0 * u + 24.0 * u + 8.0 * u + tw + 20.0 * u;
                let e = self.elev_dp(Action::Button(21)) * u;
                let rect = Rect { x, y, w: fw, h: 56.0 * u };
                ui.rect(x, y + e, fw, 56.0 * u, with_alpha(self.pal.bg, 0.7), r4(16.0 * u));
                ui.rect(x, y, fw, 56.0 * u, self.pal.primary_container, r4(16.0 * u));
                self.layer(ui, Action::Button(21), rect, 16.0 * u, ink);
                self.draw_ripples(ui, rect);
                icon_c(ui, "edit", x + 16.0 * u + 12.0 * u, y + 28.0 * u, 24.0 * u, ink);
                ui.text("Compose", x + 48.0 * u, y + 28.0 * u - style.line_h * 0.5, style);
                self.zone(Action::Button(21), rect, ink);
            }
            ComponentId::FabMenu => {
                if self.fab_open {
                    let items = [("mail", "Email"), ("call", "Call"), ("send", "Send")];
                    for (i, (ic, lb)) in items.into_iter().enumerate() {
                        let iy = y + i as f32 * 48.0 * u;
                        ui.circle(x + 20.0 * u, iy + 24.0 * u, 20.0 * u, self.pal.secondary_container);
                        icon_c(
                            ui,
                            ic,
                            x + 20.0 * u,
                            iy + 24.0 * u,
                            24.0 * u,
                            self.pal.on_secondary_container,
                        );
                        ui.text(lb, x + 52.0 * u, iy + 12.0 * u, ts(M3_BODY_LARGE, self.pal.text, u));
                    }
                }
                let fy = y + 156.0 * u;
                self.fab(
                    ui,
                    Action::FabMenu,
                    x,
                    fy,
                    56.0 * u,
                    if self.fab_open { "close" } else { "add" },
                );
            }
            ComponentId::SplitButton => {
                let h = 40.0 * u;
                let ink = self.pal.on_primary;
                let fm = if self.morph_key == 60 { self.morph.x } else { 0.0 };
                let fo = if self.morph_key == 61 { self.morph.x } else { 0.0 };
                let rm = lerp(20.0 * u, 8.0 * u, fm);
                let ro = lerp(20.0 * u, 8.0 * u, fo);
                ui.rect(x, y, 132.0 * u, h, self.pal.primary, [rm, 0.0, 0.0, rm]);
                ui.rect(x + 132.0 * u, y, 52.0 * u, h, self.pal.primary, [0.0, ro, ro, 0.0]);
                ui.rect(x + 131.0 * u, y + 10.0 * u, 1.0 * u, 20.0 * u, ink, r4(0.5 * u));
                let style = ts(M3_LABEL_LARGE, ink, u);
                let ltw = ui.text_width("Send", style.px, style.medium, style.tracking_em);
                ui.text("Send", x + (132.0 * u - ltw) * 0.5, y + h * 0.5 - style.line_h * 0.5, style);
                icon_c(ui, "expand_more", x + 158.0 * u, y + h * 0.5, 24.0 * u, ink);
                self.layer(ui, Action::Button(60), Rect { x, y, w: 132.0 * u, h }, rm, ink);
                self.draw_ripples(ui, Rect { x, y, w: 132.0 * u, h });
                self.layer(
                    ui,
                    Action::Button(61),
                    Rect { x: x + 132.0 * u, y, w: 52.0 * u, h },
                    ro,
                    ink,
                );
                self.draw_ripples(ui, Rect { x: x + 132.0 * u, y, w: 52.0 * u, h });
                self.zone(Action::Button(60), Rect { x, y, w: 132.0 * u, h }, ink);
                self.zone(Action::Button(61), Rect { x: x + 132.0 * u, y, w: 52.0 * u, h }, ink);
            }
            ComponentId::ButtonGroup => {
                let labels = ["Day", "Week", "Month"];
                let iw = 104.0 * u;
                let mut gx = x;
                for (i, label) in labels.into_iter().enumerate() {
                    if !self.is_pressed(Action::Group(i)) {
                        self.group_segment(ui, i, gx, y, label);
                    }
                    gx += iw;
                }
                let mut gx = x;
                for (i, label) in labels.into_iter().enumerate() {
                    if self.is_pressed(Action::Group(i)) {
                        self.group_segment(ui, i, gx, y, label);
                    }
                    gx += iw;
                }
            }
            ComponentId::Card => {
                let e = self.elev_dp(Action::Button(30)) * u;
                ui.rect(x, y + e, w, 140.0 * u, with_alpha(self.pal.bg, 0.7), r4(12.0 * u));
                ui.rect(x, y, w, 140.0 * u, self.pal.card_hi, r4(12.0 * u));
                ui.text("Card headline", x + 16.0 * u, y + 16.0 * u, ts(M3_TITLE_MEDIUM, self.pal.text, u));
                ui.text(
                    "Supporting copy sits here and wraps the idea.",
                    x + 16.0 * u,
                    y + 44.0 * u,
                    ts(M3_BODY_MEDIUM, self.pal.sub, u),
                );
                self.draw_button(ui, 30, x + 16.0 * u, y + 88.0 * u, BtnSpec { label: "Action", icon: None, variant: Btn::Text });
            }
            ComponentId::Checkbox => {
                let labels = ["Unchecked", "Checked", "Indeterminate"];
                for (i, label) in labels.into_iter().enumerate() {
                    self.draw_checkbox(ui, i, x, y + i as f32 * 40.0 * u, label);
                }
            }
            ComponentId::Switch => {
                let labels = ["Email alerts", "Quiet hours"];
                for (i, label) in labels.into_iter().enumerate() {
                    self.draw_switch(ui, i, x, y + i as f32 * 48.0 * u, label);
                }
            }
            ComponentId::RadioButton => {
                let labels = ["Alpha", "Beta", "Gamma"];
                for (i, label) in labels.into_iter().enumerate() {
                    self.draw_radio(ui, i, x, y + i as f32 * 40.0 * u, label);
                }
            }
            ComponentId::Slider => {
                let pct = format!("{}%", (self.slider * 100.0).round() as i32);
                ui.text(&pct, x, y, ts(M3_LABEL_LARGE, self.pal.sub, u));
                let ty = y + 56.0 * u;
                let bw = w;
                ui.rect(x, ty - 8.0 * u, bw, 16.0 * u, self.pal.secondary_container, r4(8.0 * u));
                let pressed = self.slider_drag;
                let tw = if pressed {
                    2.0 * u
                } else {
                    4.0 * u - 2.0 * u * self.slider_thumb.x
                };
                let tx = x + tw * 0.5 + (bw - tw) * self.slider;
                let gap = 3.0 * u;
                let aw = (tx - gap - x).max(0.0);
                ui.rect(x, ty - 8.0 * u, aw, 16.0 * u, self.pal.primary, r4(8.0 * u));
                let sx = x + bw - 6.0 * u;
                if sx > tx + gap {
                    ui.circle(sx, ty, 2.0 * u, self.pal.primary);
                }
                ui.rect(tx - tw * 0.5, ty - 22.0 * u, tw, 44.0 * u, self.pal.primary, r4(tw * 0.5));
                self.zone(
                    Action::Slider,
                    Rect { x, y: y + 32.0 * u, w, h: 48.0 * u },
                    self.pal.primary,
                );
            }
            ComponentId::ProgressIndicator => {
                ui.rect(x, y + 2.0 * u, w, 4.0 * u, self.pal.card_hi, r4(2.0 * u));
                ui.rect(x, y + 2.0 * u, (w * self.slider).max(4.0 * u), 4.0 * u, self.pal.primary, r4(2.0 * u));
                ui.circle(x + w - 6.0 * u, y + 4.0 * u, 2.0 * u, self.pal.primary);
                let pos = (self.t * 0.5 % 1.3) - 0.15;
                ui.rect(x + w * pos, y + 16.0 * u, w * 0.3, 4.0 * u, self.pal.primary, r4(2.0 * u));
                self.arc(ui, x + 24.0 * u, y + 52.0 * u, 18.0 * u, -90.0, self.slider * 360.0);
                let start = (self.t * 300.0) % 360.0;
                self.arc(ui, x + 84.0 * u, y + 52.0 * u, 18.0 * u, start, 270.0);
                ui.text(
                    "Linear and circular; determinate follows the slider.",
                    x + 130.0 * u,
                    y + 42.0 * u,
                    ts(M3_LABEL_MEDIUM, self.pal.sub, u),
                );
            }
            ComponentId::LoadingIndicator => {
                for i in 0..5 {
                    let ph = (self.t * 3.0 + i as f32 * 0.9).sin() * 0.5 + 0.5;
                    ui.circle(
                        x + 24.0 * u + i as f32 * 40.0 * u,
                        y + 22.0 * u,
                        (6.0 + 8.0 * ph) * u,
                        self.pal.primary,
                    );
                }
                ui.text(
                    "Expressive loading",
                    x + 232.0 * u,
                    y + 14.0 * u,
                    ts(M3_LABEL_MEDIUM, self.pal.sub, u),
                );
            }
            ComponentId::AssistChip => {
                self.draw_chip(ui, 0, x, y + 6.0 * u, "Assist", (Some("settings"), None));
            }
            ComponentId::FilterChip => {
                let lead = if self.chips[1] { Some("check") } else { None };
                self.draw_chip(ui, 1, x, y + 6.0 * u, "Filter", (lead, None));
            }
            ComponentId::InputChip => {
                self.draw_chip(ui, 2, x, y + 6.0 * u, "Input", (Some("tune"), Some("close")));
            }
            ComponentId::SuggestionChip => {
                self.draw_chip(ui, 3, x, y + 6.0 * u, "Suggest", (None, None));
            }
            ComponentId::Dialog => {
                let dw = 340.0 * u.min(w);
                let dh = 200.0 * u;
                ui.rect(x, y + 4.0 * u, dw, dh, with_alpha(self.pal.bg, 0.7), r4(28.0 * u));
                ui.rect(x, y, dw, dh, self.pal.card_hi, r4(28.0 * u));
                icon_c(ui, "delete", x + dw * 0.5, y + 32.0 * u, 24.0 * u, self.pal.error);
                text_c(
                    ui,
                    "Delete project?",
                    x + dw * 0.5,
                    y + 52.0 * u,
                    ts(M3_HEADLINE_SMALL, self.pal.text, u),
                );
                text_c(
                    ui,
                    "This action cannot be undone.",
                    x + dw * 0.5,
                    y + 92.0 * u,
                    ts(M3_BODY_MEDIUM, self.pal.sub, u),
                );
                let st = ts(M3_LABEL_LARGE, self.pal.primary, u);
                let w_c = 24.0 * u + ui.text_width("Cancel", st.px, st.medium, st.tracking_em) + 24.0 * u;
                let w_d = 24.0 * u + ui.text_width("Delete", st.px, st.medium, st.tracking_em) + 24.0 * u;
                let bx = x + dw - w_c - w_d - 12.0 * u;
                self.draw_button(ui, 31, bx, y + 148.0 * u, BtnSpec { label: "Cancel", icon: None, variant: Btn::Text });
                self.draw_button(
                    ui,
                    32,
                    bx + w_c + 12.0 * u,
                    y + 148.0 * u,
                    BtnSpec { label: "Delete", icon: None, variant: Btn::Text },
                );
            }
            ComponentId::BottomSheet => {
                ui.rect(x, y, w, 176.0 * u, self.pal.card_hi, r4(28.0 * u));
                ui.rect(x + w * 0.5 - 16.0 * u, y + 8.0 * u, 32.0 * u, 4.0 * u, self.pal.outline, r4(2.0 * u));
                let items = [("share", "Share"), ("link", "Copy link"), ("delete", "Remove")];
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let iy = y + 24.0 * u + i as f32 * 48.0 * u;
                    let a = Action::Button(70 + i as u32);
                    self.layer(ui, a, Rect { x, y: iy, w, h: 48.0 * u }, 0.0, self.pal.text);
                    self.draw_ripples(ui, Rect { x, y: iy, w, h: 48.0 * u });
                    icon_c(ui, ic, x + 28.0 * u, iy + 24.0 * u, 24.0 * u, self.pal.text);
                    ui.text(lb, x + 56.0 * u, iy + 12.0 * u, ts(M3_BODY_LARGE, self.pal.text, u));
                    self.zone(a, Rect { x, y: iy, w, h: 48.0 * u }, self.pal.text);
                }
            }
            ComponentId::NavigationBar => {
                let items = [
                    ("home", "Home"),
                    ("search", "Search"),
                    ("favorite", "Favorites"),
                    ("person", "Profile"),
                    ("settings", "Settings"),
                ];
                let cw = w / 5.0;
                ui.rect(x, y, w, 80.0 * u, self.pal.card_hi, r4(12.0 * u));
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let cx0 = x + i as f32 * cw;
                    let sel = self.nav == i;
                    let ink = if sel {
                        self.pal.on_secondary_container
                    } else {
                        self.pal.sub
                    };
                    if sel {
                        ui.rect(cx0 + cw * 0.5 - 28.0 * u, y + 12.0 * u, 56.0 * u, 32.0 * u, self.pal.secondary_container, r4(16.0 * u));
                    }
                    self.layer(ui, Action::Nav(i), Rect { x: cx0, y, w: cw, h: 80.0 * u }, 12.0 * u, self.pal.text);
                    self.draw_ripples(ui, Rect { x: cx0, y, w: cw, h: 80.0 * u });
                    icon_c(ui, ic, cx0 + cw * 0.5, y + 28.0 * u, 24.0 * u, ink);
                    let style = ts(M3_LABEL_MEDIUM, if sel { self.pal.text } else { self.pal.sub }, u);
                    let tw = ui.text_width(lb, style.px, style.medium, style.tracking_em);
                    ui.text(lb, cx0 + cw * 0.5 - tw * 0.5, y + 50.0 * u, style);
                    self.zone(Action::Nav(i), Rect { x: cx0, y, w: cw, h: 80.0 * u }, ink);
                }
            }
            ComponentId::NavigationRail => {
                let items = [("home", "Home"), ("search", "Search"), ("favorite", "Saved"), ("person", "Me")];
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let iy = y + i as f32 * 56.0 * u;
                    let sel = self.rail == i;
                    let ink = if sel {
                        self.pal.on_secondary_container
                    } else {
                        self.pal.sub
                    };
                    if sel {
                        ui.rect(x + 12.0 * u, iy + 4.0 * u, 56.0 * u, 32.0 * u, self.pal.secondary_container, r4(16.0 * u));
                    }
                    self.layer(ui, Action::Rail(i), Rect { x, y: iy, w: 80.0 * u, h: 56.0 * u }, 16.0 * u, self.pal.text);
                    self.draw_ripples(ui, Rect { x, y: iy, w: 80.0 * u, h: 56.0 * u });
                    icon_c(ui, ic, x + 40.0 * u, iy + 20.0 * u, 24.0 * u, ink);
                    let style = ts(M3_LABEL_MEDIUM, if sel { self.pal.text } else { self.pal.sub }, u);
                    let tw = ui.text_width(lb, style.px, style.medium, style.tracking_em);
                    ui.text(lb, x + 40.0 * u - tw * 0.5, iy + 38.0 * u, style);
                    self.zone(Action::Rail(i), Rect { x, y: iy, w: 80.0 * u, h: 56.0 * u }, ink);
                }
            }
            ComponentId::NavigationDrawer => {
                let dw = 280.0 * u;
                ui.rect(x, y, dw, 288.0 * u, self.pal.card_hi, r4(12.0 * u));
                ui.text("Geek Mail", x + 16.0 * u, y + 16.0 * u, ts(M3_TITLE_MEDIUM, self.pal.text, u));
                let items = [("mail", "Inbox"), ("send", "Sent"), ("edit", "Drafts"), ("delete", "Trash")];
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let iy = y + 48.0 * u + i as f32 * 56.0 * u;
                    let sel = self.drawer == i;
                    let ink = if sel {
                        self.pal.on_secondary_container
                    } else {
                        self.pal.text
                    };
                    if sel {
                        ui.rect(x + 12.0 * u, iy, dw - 24.0 * u, 56.0 * u, self.pal.secondary_container, r4(28.0 * u));
                    }
                    self.layer(ui, Action::Drawer(i), Rect { x: x + 12.0 * u, y: iy, w: dw - 24.0 * u, h: 56.0 * u }, 28.0 * u, self.pal.text);
                    self.draw_ripples(ui, Rect { x: x + 12.0 * u, y: iy, w: dw - 24.0 * u, h: 56.0 * u });
                    icon_c(ui, ic, x + 28.0 * u, iy + 28.0 * u, 24.0 * u, ink);
                    ui.text(lb, x + 56.0 * u, iy + 18.0 * u, ts(M3_LABEL_LARGE, ink, u));
                    self.zone(Action::Drawer(i), Rect { x: x + 12.0 * u, y: iy, w: dw - 24.0 * u, h: 56.0 * u }, ink);
                }
            }
            ComponentId::Scaffold => {
                let sw = 320.0 * u.min(w);
                ui.rect(x, y, sw, 64.0 * u, self.pal.card_hi, r4(12.0 * u));
                ui.rect(x, y + 32.0 * u, sw, 32.0 * u, self.pal.card_hi, r4(0.0));
                ui.text("Scaffold", x + 16.0 * u, y + 18.0 * u, ts(M3_TITLE_MEDIUM, self.pal.text, u));
                icon_c(ui, "more_vert", x + sw - 28.0 * u, y + 32.0 * u, 24.0 * u, self.pal.sub);
                ui.text("Body content lives here.", x + 16.0 * u, y + 80.0 * u, ts(M3_BODY_MEDIUM, self.pal.sub, u));
                let fy = y + 76.0 * u;
                self.fab(ui, Action::Button(80), x + sw - 72.0 * u, fy, 56.0 * u, "add");
                ui.rect(x, y + 152.0 * u, sw, 80.0 * u, self.pal.card_hi, r4(12.0 * u));
                ui.rect(x, y + 152.0 * u, sw, 40.0 * u, self.pal.card_hi, r4(0.0));
                let icons = ["home", "search", "person"];
                for (i, ic) in icons.into_iter().enumerate() {
                    icon_c(ui, ic, x + sw * 0.5 + (i as f32 - 1.0) * 64.0 * u, y + 192.0 * u, 24.0 * u, self.pal.sub);
                }
            }
            ComponentId::TopAppBar => {
                ui.rect(x, y, w, 64.0 * u, self.pal.card_hi, r4(12.0 * u));
                self.icon_button(ui, 90, x + 4.0 * u, y + 12.0 * u, "arrow_back", IconBtn::Standard);
                ui.text("Gallery", x + 56.0 * u, y + 18.0 * u, ts(M3_TITLE_LARGE, self.pal.text, u));
                self.icon_button(ui, 91, x + w - 88.0 * u, y + 12.0 * u, "search", IconBtn::Standard);
                self.icon_button(ui, 92, x + w - 44.0 * u, y + 12.0 * u, "more_vert", IconBtn::Standard);
            }
            ComponentId::SearchBar => {
                let sw = 360.0 * u.min(w);
                ui.rect(x, y + 2.0 * u, sw, 56.0 * u, with_alpha(self.pal.bg, 0.7), r4(28.0 * u));
                ui.rect(x, y, sw, 56.0 * u, self.pal.card_hi, r4(28.0 * u));
                let a = Action::Button(93);
                self.layer(ui, a, Rect { x, y, w: sw, h: 56.0 * u }, 28.0 * u, self.pal.text);
                self.draw_ripples(ui, Rect { x, y, w: sw, h: 56.0 * u });
                icon_c(ui, "search", x + 20.0 * u, y + 28.0 * u, 24.0 * u, self.pal.sub);
                ui.text("Search components", x + 52.0 * u, y + 16.0 * u, ts(M3_BODY_LARGE, self.pal.sub, u));
                self.zone(a, Rect { x, y, w: sw, h: 56.0 * u }, self.pal.text);
            }
            ComponentId::TextField => {
                let f = self.focus.x;
                ui.rect(x, y, w, 56.0 * u, self.pal.card_hi, r4(4.0 * u));
                ui.rect(x, y + 28.0 * u, w, 28.0 * u, self.pal.card_hi, r4(0.0));
                ui.text(
                    "Label",
                    x + 16.0 * u,
                    y + 6.0 * u,
                    ts(M3_BODY_SMALL, mix(self.pal.sub, self.pal.primary, f), u),
                );
                ui.text("Hello geek", x + 16.0 * u, y + 26.0 * u, ts(M3_BODY_LARGE, self.pal.text, u));
                let ih = 1.0 * u + f;
                ui.rect(x, y + 56.0 * u - ih, w, ih, mix(self.pal.outline, self.pal.primary, f), r4(0.0));
                let oy = y + 76.0 * u;
                ui.stroke(x, oy, w, 56.0 * u, self.pal.outline, r4(4.0 * u), 1.0 * u);
                ui.text("Outlined", x + 16.0 * u, oy + 6.0 * u, ts(M3_BODY_SMALL, self.pal.sub, u));
                ui.text("secret", x + 16.0 * u, oy + 26.0 * u, ts(M3_BODY_LARGE, self.pal.text, u));
                icon_c(ui, "visibility_off", x + w - 28.0 * u, oy + 28.0 * u, 24.0 * u, self.pal.sub);
            }
            ComponentId::Menu => {
                let mw = 240.0 * u;
                ui.rect(x, y + 2.0 * u, mw, 200.0 * u, with_alpha(self.pal.bg, 0.7), r4(4.0 * u));
                ui.rect(x, y, mw, 200.0 * u, self.pal.card_hi, r4(4.0 * u));
                let items = [("edit", "Rename"), ("share", "Share"), ("link", "Copy link"), ("delete", "Delete")];
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let iy = y + 4.0 * u + i as f32 * 48.0 * u;
                    let sel = self.menu_sel == i;
                    let ink = if sel {
                        self.pal.on_secondary_container
                    } else {
                        self.pal.text
                    };
                    if sel {
                        ui.rect(x + 4.0 * u, iy, mw - 8.0 * u, 48.0 * u, self.pal.secondary_container, r4(4.0 * u));
                    }
                    let rect = Rect { x: x + 4.0 * u, y: iy, w: mw - 8.0 * u, h: 48.0 * u };
                    self.layer(ui, Action::MenuItem(i), rect, 4.0 * u, self.pal.text);
                    self.draw_ripples(ui, rect);
                    icon_c(ui, ic, x + 20.0 * u, iy + 24.0 * u, 24.0 * u, ink);
                    ui.text(lb, x + 44.0 * u, iy + 14.0 * u, ts(M3_BODY_LARGE, ink, u));
                    if sel {
                        icon_c(ui, "check", x + mw - 24.0 * u, iy + 24.0 * u, 18.0 * u, ink);
                    }
                    self.zone(Action::MenuItem(i), rect, self.pal.text);
                }
            }
            ComponentId::Carousel => {
                let mut cx0 = x;
                for i in 0..3 {
                    let big = self.carousel == i;
                    let cw = if big { 200.0 * u } else { 96.0 * u };
                    let ch = if big { 128.0 * u } else { 100.0 * u };
                    let cy = y + if big { 0.0 } else { 14.0 * u };
                    let fill = if big {
                        self.pal.primary_container
                    } else {
                        self.pal.tertiary_container
                    };
                    let ink = if big {
                        self.pal.on_primary_container
                    } else {
                        self.pal.text
                    };
                    ui.rect(cx0, cy, cw, ch, fill, r4(12.0 * u));
                    icon_c(ui, "image", cx0 + cw * 0.5, cy + ch * 0.4, 24.0 * u, ink);
                    let label = format!("Item {}", i + 1);
                    ui.text(&label, cx0 + 12.0 * u, cy + ch - 32.0 * u, ts(M3_BODY_MEDIUM, ink, u));
                    cx0 += cw + 8.0 * u;
                }
                for i in 0..3 {
                    let sel = self.carousel == i;
                    let dx = x + 24.0 * u + i as f32 * 28.0 * u;
                    ui.circle(dx, y + 158.0 * u, if sel { 5.0 * u } else { 3.0 * u }, if sel { self.pal.primary } else { self.pal.outline });
                    self.zone(
                        Action::CarouselPage(i),
                        Rect { x: dx - 12.0 * u, y: y + 146.0 * u, w: 24.0 * u, h: 24.0 * u },
                        self.pal.text,
                    );
                }
            }
            ComponentId::DatePicker => {
                ui.text("October 2026", x + 12.0 * u, y + 8.0 * u, ts(M3_TITLE_MEDIUM, self.pal.text, u));
                icon_c(ui, "arrow_back", x + w - 72.0 * u, y + 20.0 * u, 24.0 * u, self.pal.sub);
                icon_c(ui, "arrow_forward", x + w - 36.0 * u, y + 20.0 * u, 24.0 * u, self.pal.sub);
                let cw = 44.0 * u;
                let x0 = x + (w - 7.0 * cw) * 0.5;
                let wd = ["M", "T", "W", "T", "F", "S", "S"];
                for (i, d) in wd.into_iter().enumerate() {
                    text_c(ui, d, x0 + i as f32 * cw + cw * 0.5, y + 44.0 * u, ts(M3_LABEL_MEDIUM, self.pal.sub, u));
                }
                for day in 0..30 {
                    let col = day % 7;
                    let row = day / 7;
                    let gx = x0 + col as f32 * cw + cw * 0.5;
                    let gy = y + 72.0 * u + row as f32 * 40.0 * u + 20.0 * u;
                    let sel = day == 12;
                    let today = day == 25;
                    if sel {
                        ui.circle(gx, gy, 20.0 * u, self.pal.primary);
                    } else if today {
                        ui.stroke(
                            gx - 18.0 * u,
                            gy - 18.0 * u,
                            36.0 * u,
                            36.0 * u,
                            self.pal.outline,
                            r4(18.0 * u),
                            1.0 * u,
                        );
                    }
                    let label = format!("{}", day + 1);
                    text_c(
                        ui,
                        &label,
                        gx,
                        gy - 10.0 * u,
                        ts(M3_LABEL_LARGE, if sel { self.pal.on_primary } else { self.pal.text }, u),
                    );
                }
            }
            ComponentId::TimePicker => {
                let cx = x + 100.0 * u;
                let cy = y + 100.0 * u;
                ui.circle(cx, cy, 88.0 * u, self.pal.card_hi);
                for k in 0..12 {
                    let ang = (k as f32 * 30.0f32 - 90.0f32).to_radians();
                    ui.circle(cx + 74.0 * u * ang.cos(), cy + 74.0 * u * ang.sin(), 2.0 * u, self.pal.sub);
                }
                let nums = [("12", -90.0f32), ("3", 0.0), ("6", 90.0), ("9", 180.0)];
                for (n, deg) in nums.into_iter() {
                    let ang = deg.to_radians();
                    text_c(
                        ui,
                        n,
                        cx + 56.0 * u * ang.cos(),
                        cy + 56.0 * u * ang.sin() - 10.0 * u,
                        ts(M3_LABEL_LARGE, self.pal.text, u),
                    );
                }
                ui.rect(cx - 2.0 * u, cy - 56.0 * u, 4.0 * u, 56.0 * u, self.pal.primary, r4(2.0 * u));
                let hand = 306.0f32.to_radians();
                for k in 1..=9 {
                    let rr = k as f32 * 4.0 * u;
                    ui.circle(cx + rr * hand.cos(), cy + rr * hand.sin(), 2.2 * u, self.pal.primary);
                }
                ui.circle(cx, cy, 5.0 * u, self.pal.primary);
                ui.text("10:24", x + 212.0 * u, y + 70.0 * u, ts(M3_DISPLAY_SMALL, self.pal.text, u));
                ui.text("AM", x + 216.0 * u, y + 122.0 * u, ts(M3_LABEL_LARGE, self.pal.sub, u));
            }
            ComponentId::Tooltip => {
                ui.rect(x + 40.0 * u, y, 132.0 * u, 32.0 * u, self.pal.inverse, r4(4.0 * u));
                text_c(
                    ui,
                    "Plain tooltip",
                    x + 106.0 * u,
                    y + 8.0 * u,
                    ts(M3_BODY_SMALL, self.pal.on_inverse, u),
                );
                ui.rect(x + 40.0 * u, y + 44.0 * u, 280.0 * u, 64.0 * u, self.pal.card_hi, r4(12.0 * u));
                ui.stroke(
                    x + 40.0 * u,
                    y + 44.0 * u,
                    280.0 * u,
                    64.0 * u,
                    self.pal.outline_variant,
                    r4(12.0 * u),
                    1.0 * u,
                );
                ui.text("Rich tooltip", x + 56.0 * u, y + 52.0 * u, ts(M3_TITLE_MEDIUM, self.pal.text, u));
                ui.text("Long press an item to show this.", x + 56.0 * u, y + 76.0 * u, ts(M3_BODY_SMALL, self.pal.sub, u));
            }
            ComponentId::Snackbar => {
                ui.rect(x, y + 8.0 * u, w, 48.0 * u, self.pal.inverse, r4(4.0 * u));
                let st = ts(M3_LABEL_LARGE, self.pal.inverse_primary, u);
                let tw = ui.text_width("Undo", st.px, st.medium, st.tracking_em);
                let a = Action::Button(96);
                let rect = Rect { x: x + w - 16.0 * u - tw - 8.0 * u, y: y + 8.0 * u, w: tw + 16.0 * u, h: 48.0 * u };
                self.layer(ui, a, rect, 4.0 * u, self.pal.inverse_primary);
                self.draw_ripples(ui, rect);
                ui.text("Message sent", x + 16.0 * u, y + 22.0 * u, ts(M3_BODY_MEDIUM, self.pal.on_inverse, u));
                ui.text("Undo", x + w - 16.0 * u - tw, y + 22.0 * u, st);
                self.zone(a, rect, self.pal.inverse_primary);
            }
            ComponentId::Badge => {
                icon_c(ui, "notifications", x + 24.0 * u, y + 24.0 * u, 24.0 * u, self.pal.text);
                ui.circle(x + 38.0 * u, y + 12.0 * u, 3.0 * u, self.pal.error);
                icon_c(ui, "person", x + 72.0 * u, y + 24.0 * u, 24.0 * u, self.pal.text);
                ui.circle(x + 86.0 * u, y + 12.0 * u, 8.0 * u, self.pal.error);
                text_c(ui, "3", x + 86.0 * u, y + 4.0 * u, ts(M3_LABEL_SMALL, self.pal.on_error, u));
                ui.text("Badges sit on the top-right corner.", x + 116.0 * u, y + 18.0 * u, ts(M3_BODY_MEDIUM, self.pal.sub, u));
            }
            ComponentId::ListItem => {
                let rows = [
                    ("person", "Single line item", "", 56.0f32),
                    ("history", "Two line item", "Supporting text follows the headline.", 72.0),
                    ("description", "Three line item", "First supporting line of copy.", 88.0),
                ];
                let mut ry = y;
                for (i, (ic, head, support, h)) in rows.into_iter().enumerate() {
                    let hh = h * u;
                    let a = Action::Button(110 + i as u32);
                    self.layer(ui, a, Rect { x, y: ry, w, h: hh }, 0.0, self.pal.text);
                    self.draw_ripples(ui, Rect { x, y: ry, w, h: hh });
                    icon_c(ui, ic, x + 28.0 * u, ry + hh * 0.5, 24.0 * u, self.pal.text);
                    ui.text(head, x + 56.0 * u, ry + if h >= 88.0 { 12.0 } else { 16.0 } * u, ts(M3_BODY_LARGE, self.pal.text, u));
                    if !support.is_empty() {
                        ui.text(support, x + 56.0 * u, ry + if h >= 88.0 { 36.0 } else { 40.0 } * u, ts(M3_BODY_MEDIUM, self.pal.sub, u));
                    }
                    if h >= 88.0 {
                        ui.text(
                            "Second supporting line of copy.",
                            x + 56.0 * u,
                            ry + 56.0 * u,
                            ts(M3_BODY_MEDIUM, self.pal.sub, u),
                        );
                    }
                    let tr = "12:00";
                    let style = ts(M3_LABEL_SMALL, self.pal.sub, u);
                    let tw = ui.text_width(tr, style.px, style.medium, style.tracking_em);
                    ui.text(tr, x + w - 16.0 * u - tw, ry + if h >= 72.0 { 16.0 } else { 20.0 } * u, style);
                    self.zone(a, Rect { x, y: ry, w, h: hh }, self.pal.text);
                    ry += hh;
                }
            }
            ComponentId::Tabs => {
                let items = [("code", "Code"), ("terminal", "Build"), ("send", "Ship"), ("description", "Docs")];
                let tw2 = w / 4.0;
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let tx = x + i as f32 * tw2;
                    let sel = self.tabs == i;
                    let col = if sel { self.pal.primary } else { self.pal.sub };
                    icon_c(ui, ic, tx + tw2 * 0.5, y + 20.0 * u, 24.0 * u, col);
                    text_c(ui, lb, tx + tw2 * 0.5, y + 36.0 * u, ts(M3_TITLE_SMALL, col, u));
                    if sel {
                        ui.rect(tx + 24.0 * u, y + 61.0 * u, tw2 - 48.0 * u, 3.0 * u, self.pal.primary, r4(1.5 * u));
                    }
                    let rect = Rect { x: tx, y, w: tw2, h: 64.0 * u };
                    self.layer(ui, Action::Tab(i), rect, 0.0, self.pal.text);
                    self.draw_ripples(ui, rect);
                    self.zone(Action::Tab(i), rect, self.pal.text);
                }
            }
            ComponentId::SegmentedButton => {
                let labels = ["On", "Off", "Auto"];
                let sw = w / 3.0;
                ui.stroke(x, y, w, 40.0 * u, self.pal.outline, r4(20.0 * u), 1.0 * u);
                for (i, lb) in labels.into_iter().enumerate() {
                    let sx = x + i as f32 * sw;
                    let sel = self.segmented == i;
                    if sel {
                        let end = 19.0 * u;
                        let fill_r = if i == 0 {
                            [end, 0.0, 0.0, end]
                        } else if i == labels.len() - 1 {
                            [0.0, end, end, 0.0]
                        } else {
                            r4(0.0)
                        };
                        ui.rect(
                            sx + 1.0 * u,
                            y + 1.0 * u,
                            sw - 2.0 * u,
                            38.0 * u,
                            self.pal.secondary_container,
                            fill_r,
                        );
                    }
                    if i > 0 && self.segmented != i && self.segmented != i - 1 {
                        ui.rect(sx - 0.5 * u, y + 8.0 * u, 1.0 * u, 24.0 * u, self.pal.outline, r4(0.0));
                    }
                    let ink = if sel {
                        self.pal.on_secondary_container
                    } else {
                        self.pal.text
                    };
                    let style = ts(M3_LABEL_LARGE, ink, u);
                    let ltw = ui.text_width(lb, style.px, style.medium, style.tracking_em);
                    let icon_w = if sel { 26.0 * u } else { 0.0 };
                    let start = sx + (sw - ltw - icon_w) * 0.5;
                    if sel {
                        icon_c(ui, "check", start + 9.0 * u, y + 20.0 * u, 18.0 * u, ink);
                    }
                    ui.text(lb, start + icon_w, y + 20.0 * u - style.line_h * 0.5, style);
                    let rect = Rect { x: sx, y, w: sw, h: 40.0 * u };
                    self.layer(ui, Action::Segmented(i), rect, 0.0, self.pal.text);
                    self.draw_ripples(ui, rect);
                    self.zone(Action::Segmented(i), rect, self.pal.text);
                }
            }
            ComponentId::Divider => {
                ui.rect(x, y + 12.0 * u, w, 1.0 * u, self.pal.outline_variant, r4(0.5 * u));
            }
            ComponentId::PullToRefresh => {
                let start = (self.t * 240.0) % 360.0;
                self.arc(ui, x + 28.0 * u, y + 24.0 * u, 14.0 * u, start, 270.0);
                ui.text("Pull down to refresh", x + 64.0 * u, y + 14.0 * u, ts(M3_BODY_MEDIUM, self.pal.sub, u));
            }
            ComponentId::SwipeToDismiss => {
                ui.rect(x, y + 4.0 * u, w, 56.0 * u, self.pal.error, r4(12.0 * u));
                icon_c(ui, "delete", x + w - 32.0 * u, y + 32.0 * u, 24.0 * u, self.pal.on_error);
                ui.text("Delete", x + w - 108.0 * u, y + 22.0 * u, ts(M3_LABEL_LARGE, self.pal.on_error, u));
                let fw = w - 90.0 * u;
                ui.rect(x, y + 6.0 * u, fw, 56.0 * u, with_alpha(self.pal.bg, 0.7), r4(12.0 * u));
                ui.rect(x, y + 4.0 * u, fw, 56.0 * u, self.pal.card_hi, r4(12.0 * u));
                ui.text("Swipe me away", x + 16.0 * u, y + 22.0 * u, ts(M3_BODY_LARGE, self.pal.text, u));
                icon_c(ui, "mail", x + fw - 32.0 * u, y + 32.0 * u, 24.0 * u, self.pal.sub);
            }
            ComponentId::Toolbar => {
                let icons = ["edit", "palette", "brush", "more_vert"];
                let tw3 = 4.0 * 40.0 * u + 8.0 * u;
                ui.rect(x, y + 2.0 * u, tw3, 52.0 * u, with_alpha(self.pal.bg, 0.7), r4(26.0 * u));
                ui.rect(x, y, tw3, 52.0 * u, self.pal.card_hi, r4(26.0 * u));
                for (i, ic) in icons.into_iter().enumerate() {
                    self.icon_button(ui, 40 + i as u32, x + 4.0 * u + i as f32 * 40.0 * u, y + 6.0 * u, ic, IconBtn::Standard);
                }
            }
        }
    }
}

pub struct GalleryApp {
    gallery: Gallery,
    ui: Option<UiInner>,
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
        }
    }

    pub fn resume(&mut self, window: Arc<Window>) -> bool {
        use wgpu::{
            CompositeAlphaMode, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
            InstanceDescriptor, MemoryHints, PowerPreference, PresentMode, RequestAdapterOptions,
            TextureFormat, TextureUsages, Trace,
        };
        use log::{error, info};
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
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(ui) = self.ui.as_mut() {
            ui.config.width = width.max(1);
            ui.config.height = height.max(1);
            ui.surface.configure(&ui.device, &ui.config);
            ui.renderer
                .set_screen(
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
        self.gallery.step();
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
        self.gallery
            .draw(&mut ui.renderer, ui.config.width as f32, ui.config.height as f32);
        ui.renderer.flush(&ui.device, &ui.queue, &view, self.gallery.bg());
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
