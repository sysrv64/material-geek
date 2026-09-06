use mg_components::ComponentId;
use mg_motion::{DynamicSpring, MotionScheme, Tempo, Track};
use mg_render::{TextStyle, UiRenderer};
use std::sync::Arc;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

fn srgb_to_linear(byte: u8) -> f32 {
    let f = byte as f32 / 255.0;
    if f <= 0.04045 {
        f / 12.92
    } else {
        ((f + 0.055) / 1.055).powf(2.4)
    }
}

fn argb(argb: u32) -> [f32; 4] {
    [
        srgb_to_linear(((argb >> 16) & 0xFF) as u8),
        srgb_to_linear(((argb >> 8) & 0xFF) as u8),
        srgb_to_linear((argb & 0xFF) as u8),
        1.0,
    ]
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
enum Action {
    Switch(usize),
    Check(usize),
    Radio(usize),
    Chip(usize),
    Tab(usize),
    Nav(usize),
    Rail(usize),
    Segmented(usize),
    MenuItem(usize),
    CarouselPage(usize),
    FabMenu,
    Button(u32),
    Slider,
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
        }
    }
}

pub struct Gallery {
    pal: Palette,
    scroll: f32,
    content_h: f32,
    view_h: f32,
    down: Option<(f32, f32)>,
    down_moved: bool,
    drag_last_y: f32,
    slider_drag: bool,
    switches: [bool; 2],
    checks: [u8; 3],
    radio: usize,
    chips: [bool; 4],
    tabs: usize,
    nav: usize,
    rail: usize,
    segmented: usize,
    menu_sel: usize,
    carousel: usize,
    fab_open: bool,
    slider: f32,
    press_id: u32,
    press: DynamicSpring,
    load: DynamicSpring,
    frames: u64,
    zones: Vec<(Action, Rect)>,
}

impl Gallery {
    pub fn new() -> Self {
        let fast = MotionScheme::expressive().spec(Tempo::Fast, Track::Effects);
        let slow = MotionScheme::expressive().spec(Tempo::Slow, Track::Spatial);
        Self {
            pal: Palette::dark(),
            scroll: 0.0,
            content_h: 1000.0,
            view_h: 1000.0,
            down: None,
            down_moved: false,
            drag_last_y: 0.0,
            slider_drag: false,
            switches: [true, false],
            checks: [0, 1, 2],
            radio: 0,
            chips: [false, true, false, false],
            tabs: 0,
            nav: 0,
            rail: 0,
            segmented: 0,
            menu_sel: 1,
            carousel: 0,
            fab_open: false,
            slider: 0.6,
            press_id: 0,
            press: DynamicSpring::new(0.0, 0.0, fast),
            load: DynamicSpring::new(0.0, 1.0, slow),
            frames: 0,
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
        self.frames += 1;
        self.press.step(1.0 / 60.0);
        self.load.step(1.0 / 60.0);
        if self.load.settled() {
            self.load.retarget(if self.load.target >= 1.0 { 0.0 } else { 1.0 });
        }
    }

    pub fn time(&self) -> f32 {
        self.frames as f32 / 60.0
    }

    pub fn scroll_by(&mut self, dy: f32) {
        let max = (self.content_h - self.view_h).max(0.0);
        self.scroll = (self.scroll + dy).clamp(0.0, max);
    }

    pub fn down(&mut self, x: f32, y: f32) {
        self.down = Some((x, y));
        self.down_moved = false;
        self.drag_last_y = y;
        self.slider_drag = self
            .zones
            .iter()
            .any(|(a, r)| matches!(a, Action::Slider) && r.contains(x, y));
        if self.slider_drag {
            self.drag_slider_to(x);
            return;
        }
        for (a, r) in self.zones.clone() {
            if r.contains(x, y) {
                if let Action::Button(id) = a {
                    self.press_id = id;
                    self.press.retarget(1.0);
                }
                break;
            }
        }
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        if let Some((dx, _)) = self.down {
            if (x - dx).abs() > 8.0 {
                self.down_moved = true;
            }
            if self.slider_drag {
                self.drag_slider_to(x);
            } else {
                let dy = self.drag_last_y - y;
                if dy.abs() > 1.0 {
                    self.down_moved = true;
                }
                self.scroll_by(dy);
                self.drag_last_y = y;
            }
        }
    }

    pub fn up(&mut self, x: f32, y: f32) {
        if self.slider_drag {
            self.slider_drag = false;
            self.down = None;
            return;
        }
        let tapped = !self.down_moved;
        self.down = None;
        self.press.retarget(0.0);
        if !tapped {
            return;
        }
        for (a, r) in self.zones.clone() {
            if r.contains(x, y) {
                self.activate(a);
                break;
            }
        }
    }

    fn drag_slider_to(&mut self, x: f32) {
        for (a, r) in self.zones.clone() {
            if matches!(a, Action::Slider) && x >= r.x - 60.0 && x <= r.x + r.w + 60.0 {
                self.slider = ((x - r.x) / r.w).clamp(0.0, 1.0);
                break;
            }
        }
    }

    fn activate(&mut self, action: Action) {
        match action {
            Action::Switch(i) => self.switches[i] = !self.switches[i],
            Action::Check(i) => self.checks[i] = (self.checks[i] + 1) % 3,
            Action::Radio(i) => self.radio = i,
            Action::Chip(i) => self.chips[i] = !self.chips[i],
            Action::Tab(i) => self.tabs = i,
            Action::Nav(i) => self.nav = i,
            Action::Rail(i) => self.rail = i,
            Action::Segmented(i) => self.segmented = i,
            Action::MenuItem(i) => self.menu_sel = i,
            Action::CarouselPage(i) => self.carousel = i,
            Action::FabMenu => self.fab_open = !self.fab_open,
            Action::Button(_) => {}
            Action::Slider => {}
        }
    }

    fn zone(&mut self, action: Action, rect: Rect) {
        self.zones.push((action, rect));
    }

    pub fn draw(&mut self, ui: &mut UiRenderer, width: f32, height: f32) {
        self.view_h = height;
        self.zones.clear();
        let u = Self::unit(width);
        let pad = 16.0 * u;
        let mut y = 24.0 * u - self.scroll;
        ui.text(
            "material-geek",
            pad,
            y,
            TextStyle {
                px: 22.0 * u,
                medium: true,
                color: self.pal.text,
                tracking_em: 0.0,
            },
        );
        ui.text(
            "41 components - dark expressive - drag to scroll, tap to toggle",
            pad,
            y + 30.0 * u,
            TextStyle {
                px: 11.0 * u,
                medium: false,
                color: self.pal.sub,
                tracking_em: 0.0,
            },
        );
        y += 62.0 * u;
        for id in ComponentId::ALL {
            let h = Self::body_height(id, u);
            let name = format!("{id:?}");
            ui.text(
                &name,
                pad,
                y,
                TextStyle {
                    px: 11.0 * u,
                    medium: true,
                    color: self.pal.sub,
                    tracking_em: 0.1,
                },
            );
            let card_y = y + 20.0 * u;
            ui.rect(
                pad * 0.5,
                card_y,
                width - pad,
                h + 24.0 * u,
                self.pal.card,
                12.0 * u,
            );
            self.draw_body(ui, id, pad, card_y + 12.0 * u, width - pad * 2.0, u);
            y = card_y + h + 24.0 * u + 14.0 * u;
        }
        self.content_h = y + self.scroll + 40.0 * u;
        self.scroll_by(0.0);
    }

    fn body_height(id: ComponentId, u: f32) -> f32 {
        match id {
            ComponentId::Button => 64.0 * u,
            ComponentId::IconButton => 64.0 * u,
            ComponentId::Fab => 76.0 * u,
            ComponentId::ExtendedFab => 64.0 * u,
            ComponentId::FabMenu => 190.0 * u,
            ComponentId::SplitButton => 56.0 * u,
            ComponentId::ButtonGroup => 56.0 * u,
            ComponentId::Card => 150.0 * u,
            ComponentId::Checkbox => 150.0 * u,
            ComponentId::Switch => 110.0 * u,
            ComponentId::RadioButton => 150.0 * u,
            ComponentId::Slider => 70.0 * u,
            ComponentId::ProgressIndicator => 120.0 * u,
            ComponentId::LoadingIndicator => 70.0 * u,
            ComponentId::AssistChip => 52.0 * u,
            ComponentId::FilterChip => 52.0 * u,
            ComponentId::InputChip => 52.0 * u,
            ComponentId::SuggestionChip => 52.0 * u,
            ComponentId::Dialog => 210.0 * u,
            ComponentId::BottomSheet => 190.0 * u,
            ComponentId::NavigationBar => 76.0 * u,
            ComponentId::NavigationRail => 200.0 * u,
            ComponentId::NavigationDrawer => 220.0 * u,
            ComponentId::Scaffold => 240.0 * u,
            ComponentId::TopAppBar => 110.0 * u,
            ComponentId::SearchBar => 64.0 * u,
            ComponentId::TextField => 160.0 * u,
            ComponentId::Menu => 220.0 * u,
            ComponentId::Carousel => 170.0 * u,
            ComponentId::DatePicker => 320.0 * u,
            ComponentId::TimePicker => 190.0 * u,
            ComponentId::Tooltip => 120.0 * u,
            ComponentId::Snackbar => 70.0 * u,
            ComponentId::Badge => 64.0 * u,
            ComponentId::ListItem => 180.0 * u,
            ComponentId::Tabs => 60.0 * u,
            ComponentId::SegmentedButton => 60.0 * u,
            ComponentId::Divider => 30.0 * u,
            ComponentId::PullToRefresh => 70.0 * u,
            ComponentId::SwipeToDismiss => 70.0 * u,
            ComponentId::Toolbar => 64.0 * u,
        }
    }

    fn press_morph(&self, id: u32, base: f32, u: f32) -> f32 {
        if self.press_id == id {
            base + (4.0 * u - base) * self.press.x
        } else {
            base
        }
    }

    fn button(
        &mut self,
        ui: &mut UiRenderer,
        id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        label: &str,
        style: u8,
        u: f32,
    ) {
        let morph = self.press_morph(id, h * 0.5, u);
        let (fill, ink, outlined) = match style {
            0 => (self.pal.primary, self.pal.on_primary, false),
            1 => (self.pal.secondary_container, self.pal.on_secondary_container, false),
            2 => (self.pal.card, self.pal.primary, true),
            _ => (self.pal.card, self.pal.primary, false),
        };
        ui.rect(x, y, w, h, fill, morph);
        if outlined {
            self.frame(ui, x, y, w, h, morph, 1.5 * u, self.pal.outline);
        }
        let tw = ui.text_width(label, 14.0 * u, true, 0.1);
        ui.text(
            label,
            x + (w - tw) * 0.5,
            y + (h - 20.0 * u) * 0.5,
            TextStyle {
                px: 14.0 * u,
                medium: true,
                color: ink,
                tracking_em: 0.1,
            },
        );
        self.zone(Action::Button(id), Rect { x, y, w, h });
    }

    fn frame(
        &mut self,
        ui: &mut UiRenderer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
        t: f32,
        color: [f32; 4],
    ) {
        ui.rect(x, y, w, t, color, r);
        ui.rect(x, y + h - t, w, t, color, r);
        ui.rect(x, y, t, h, color, r);
        ui.rect(x + w - t, y, t, h, color, r);
    }

    fn draw_body(
        &mut self,
        ui: &mut UiRenderer,
        id: ComponentId,
        x: f32,
        y: f32,
        w: f32,
        u: f32,
    ) {
        match id {
            ComponentId::Button => {
                self.button(ui, 1, x, y, 120.0 * u, 44.0 * u, "Filled", 0, u);
                self.button(ui, 2, x + 130.0 * u, y, 120.0 * u, 44.0 * u, "Tonal", 1, u);
                self.button(ui, 3, x + 260.0 * u, y, 130.0 * u, 44.0 * u, "Outlined", 2, u);
            }
            ComponentId::IconButton => {
                for (i, glyph) in ["+", "-", "x"].iter().enumerate() {
                    let cx = x + 20.0 * u + i as f32 * 64.0 * u;
                    let fill = if i == 0 { self.pal.primary } else { self.pal.card_hi };
                    ui.circle(cx + 22.0 * u, y + 22.0 * u, 22.0 * u, fill);
                    let tw = ui.text_width(glyph, 20.0 * u, true, 0.0);
                    ui.text(
                        glyph,
                        cx + 22.0 * u - tw * 0.5,
                        y + 22.0 * u - 13.0 * u,
                        TextStyle {
                            px: 20.0 * u,
                            medium: true,
                            color: if i == 0 { self.pal.on_primary } else { self.pal.text },
                            tracking_em: 0.0,
                        },
                    );
                    self.zone(
                        Action::Button(10 + i as u32),
                        Rect { x: cx, y, w: 44.0 * u, h: 44.0 * u },
                    );
                }
            }
            ComponentId::Fab => {
                ui.circle(x + 34.0 * u, y + 34.0 * u, 30.0 * u, self.pal.primary_container);
                let tw = ui.text_width("+", 26.0 * u, false, 0.0);
                ui.text(
                    "+",
                    x + 34.0 * u - tw * 0.5,
                    y + 34.0 * u - 17.0 * u,
                    TextStyle {
                        px: 26.0 * u,
                        medium: false,
                        color: self.pal.on_primary_container,
                        tracking_em: 0.0,
                    },
                );
                ui.text(
                    "FAB - primary action",
                    x + 80.0 * u,
                    y + 22.0 * u,
                    TextStyle {
                        px: 14.0 * u,
                        medium: false,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
                self.zone(
                    Action::Button(20),
                    Rect { x, y, w: 68.0 * u, h: 68.0 * u },
                );
            }
            ComponentId::ExtendedFab => {
                ui.rect(x, y, 170.0 * u, 52.0 * u, self.pal.primary_container, 16.0 * u);
                ui.text(
                    "+",
                    x + 20.0 * u,
                    y + 12.0 * u,
                    TextStyle {
                        px: 22.0 * u,
                        medium: false,
                        color: self.pal.on_primary_container,
                        tracking_em: 0.0,
                    },
                );
                ui.text(
                    "Compose",
                    x + 48.0 * u,
                    y + 15.0 * u,
                    TextStyle {
                        px: 15.0 * u,
                        medium: true,
                        color: self.pal.on_primary_container,
                        tracking_em: 0.1,
                    },
                );
                self.zone(
                    Action::Button(21),
                    Rect { x, y, w: 170.0 * u, h: 52.0 * u },
                );
            }
            ComponentId::FabMenu => {
                if self.fab_open {
                    for i in 0..3 {
                        let by = y + (2 - i) as f32 * 56.0 * u;
                        ui.circle(x + 30.0 * u, by + 22.0 * u, 20.0 * u, self.pal.secondary_container);
                        ui.text(
                            ["Mail", "Chat", "Call"][i],
                            x + 60.0 * u,
                            by + 14.0 * u,
                            TextStyle {
                                px: 13.0 * u,
                                medium: false,
                                color: self.pal.text,
                                tracking_em: 0.0,
                            },
                        );
                    }
                }
                ui.circle(x + 30.0 * u, y + 150.0 * u, 26.0 * u, self.pal.primary);
                let tw = ui.text_width(if self.fab_open { "x" } else { "+" }, 24.0 * u, false, 0.0);
                ui.text(
                    if self.fab_open { "x" } else { "+" },
                    x + 30.0 * u - tw * 0.5,
                    y + 150.0 * u - 16.0 * u,
                    TextStyle {
                        px: 24.0 * u,
                        medium: false,
                        color: self.pal.on_primary,
                        tracking_em: 0.0,
                    },
                );
                ui.text(
                    if self.fab_open { "tap to close" } else { "tap to expand" },
                    x + 66.0 * u,
                    y + 142.0 * u,
                    TextStyle {
                        px: 13.0 * u,
                        medium: false,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
                self.zone(
                    Action::FabMenu,
                    Rect { x, y: y + 124.0 * u, w: 52.0 * u, h: 52.0 * u },
                );
            }
            ComponentId::SplitButton => {
                ui.rect(x, y, 150.0 * u, 44.0 * u, self.pal.primary, 22.0 * u);
                ui.rect(x + 150.0 * u, y, 52.0 * u, 44.0 * u, self.pal.primary_container, 0.0);
                ui.rect(x + 176.0 * u, y, 26.0 * u, 44.0 * u, self.pal.primary_container, 13.0 * u);
                ui.text(
                    "Share",
                    x + 24.0 * u,
                    y + 12.0 * u,
                    TextStyle {
                        px: 14.0 * u,
                        medium: true,
                        color: self.pal.on_primary,
                        tracking_em: 0.1,
                    },
                );
                ui.text(
                    "v",
                    x + 168.0 * u,
                    y + 12.0 * u,
                    TextStyle {
                        px: 14.0 * u,
                        medium: true,
                        color: self.pal.on_primary_container,
                        tracking_em: 0.0,
                    },
                );
                self.zone(Action::Button(22), Rect { x, y, w: 150.0 * u, h: 44.0 * u });
                self.zone(Action::Button(23), Rect { x: x + 150.0 * u, y, w: 52.0 * u, h: 44.0 * u });
            }
            ComponentId::ButtonGroup => {
                let labels = ["Day", "Week", "Month"];
                for i in 0..3 {
                    let bx = x + i as f32 * 110.0 * u;
                    let sel = self.segmented == i;
                    ui.rect(
                        bx,
                        y,
                        110.0 * u,
                        44.0 * u,
                        if sel { self.pal.secondary_container } else { self.pal.card_hi },
                        4.0 * u,
                    );
                    let tw = ui.text_width(labels[i], 14.0 * u, true, 0.0);
                    ui.text(
                        labels[i],
                        bx + (110.0 * u - tw) * 0.5,
                        y + 12.0 * u,
                        TextStyle {
                            px: 14.0 * u,
                            medium: true,
                            color: if sel { self.pal.on_secondary_container } else { self.pal.text },
                            tracking_em: 0.0,
                        },
                    );
                    self.zone(
                        Action::Segmented(i),
                        Rect { x: bx, y, w: 110.0 * u, h: 44.0 * u },
                    );
                }
            }
            ComponentId::Card => {
                ui.rect(x, y, w, 140.0 * u, self.pal.card_hi, 12.0 * u);
                ui.text(
                    "Card headline",
                    x + 16.0 * u,
                    y + 12.0 * u,
                    TextStyle {
                        px: 16.0 * u,
                        medium: true,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
                ui.text(
                    "Supporting copy sits here and wraps the idea.",
                    x + 16.0 * u,
                    y + 40.0 * u,
                    TextStyle {
                        px: 13.0 * u,
                        medium: false,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
                self.button(ui, 30, x + 16.0 * u, y + 84.0 * u, 110.0 * u, 40.0 * u, "Action", 3, u);
            }
            ComponentId::Checkbox => {
                let labels = ["Off", "On", "Mixed"];
                for i in 0..3 {
                    let cy = y + i as f32 * 48.0 * u;
                    let st = self.checks[i];
                    if st == 0 {
                        self.frame(ui, x, cy + 4.0 * u, 26.0 * u, 26.0 * u, 5.0 * u, 2.0 * u, self.pal.outline);
                    } else {
                        ui.rect(x, cy + 4.0 * u, 26.0 * u, 26.0 * u, self.pal.primary, 5.0 * u);
                        if st == 1 {
                            ui.rect(x + 6.0 * u, cy + 10.0 * u, 14.0 * u, 14.0 * u, self.pal.on_primary, 3.0 * u);
                        } else {
                            ui.rect(x + 6.0 * u, cy + 15.0 * u, 14.0 * u, 4.0 * u, self.pal.on_primary, 2.0 * u);
                        }
                    }
                    ui.text(
                        labels[i],
                        x + 38.0 * u,
                        cy + 6.0 * u,
                        TextStyle {
                            px: 14.0 * u,
                            medium: false,
                            color: self.pal.text,
                            tracking_em: 0.0,
                        },
                    );
                    self.zone(Action::Check(i), Rect { x, y: cy, w: 200.0 * u, h: 40.0 * u });
                }
            }
            ComponentId::Switch => {
                for i in 0..2 {
                    let cy = y + i as f32 * 52.0 * u;
                    let on = self.switches[i];
                    ui.rect(
                        x,
                        cy + 6.0 * u,
                        52.0 * u,
                        32.0 * u,
                        if on { self.pal.primary } else { self.pal.outline_variant },
                        16.0 * u,
                    );
                    ui.circle(
                        x + if on { 36.0 * u } else { 16.0 * u },
                        cy + 22.0 * u,
                        if on { 12.0 * u } else { 8.0 * u },
                        if on { self.pal.on_primary } else { self.pal.outline },
                    );
                    ui.text(
                        if on { "On" } else { "Off" },
                        x + 64.0 * u,
                        cy + 10.0 * u,
                        TextStyle {
                            px: 14.0 * u,
                            medium: false,
                            color: self.pal.text,
                            tracking_em: 0.0,
                        },
                    );
                    self.zone(Action::Switch(i), Rect { x, y: cy, w: 160.0 * u, h: 44.0 * u });
                }
            }
            ComponentId::RadioButton => {
                let labels = ["Alpha", "Beta", "Gamma"];
                for i in 0..3 {
                    let cy = y + i as f32 * 48.0 * u;
                    ui.circle(x + 15.0 * u, cy + 17.0 * u, 13.0 * u, self.pal.card);
                    self.frame(ui, x + 2.0 * u, cy + 4.0 * u, 26.0 * u, 26.0 * u, 13.0 * u, 2.0 * u, self.pal.outline);
                    if self.radio == i {
                        ui.circle(x + 15.0 * u, cy + 17.0 * u, 7.0 * u, self.pal.primary);
                    }
                    ui.text(
                        labels[i],
                        x + 40.0 * u,
                        cy + 6.0 * u,
                        TextStyle {
                            px: 14.0 * u,
                            medium: false,
                            color: self.pal.text,
                            tracking_em: 0.0,
                        },
                    );
                    self.zone(Action::Radio(i), Rect { x, y: cy, w: 200.0 * u, h: 40.0 * u });
                }
            }
            ComponentId::Slider => {
                let bw = w - 40.0 * u;
                ui.rect(x + 20.0 * u, y + 30.0 * u, bw, 5.0 * u, self.pal.outline_variant, 2.5 * u);
                ui.rect(x + 20.0 * u, y + 30.0 * u, bw * self.slider, 5.0 * u, self.pal.primary, 2.5 * u);
                ui.circle(x + 20.0 * u + bw * self.slider, y + 32.5 * u, 12.0 * u, self.pal.primary);
                let val = format!("{:.0}", self.slider * 100.0);
                ui.text(
                    &val,
                    x + 20.0 * u,
                    y,
                    TextStyle {
                        px: 12.0 * u,
                        medium: true,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
                self.zone(Action::Slider, Rect { x, y, w, h: 60.0 * u });
            }
            ComponentId::ProgressIndicator => {
                ui.rect(x, y + 8.0 * u, w, 6.0 * u, self.pal.outline_variant, 3.0 * u);
                ui.rect(x, y + 8.0 * u, w * 0.6, 6.0 * u, self.pal.primary, 3.0 * u);
                for i in 0..16 {
                    let ph = (self.time() * 2.0 + i as f32 * 0.4).sin() * 4.0 * u;
                    ui.circle(x + 12.0 * u + i as f32 * 24.0 * u, y + 44.0 * u + ph, 4.0 * u, self.pal.primary);
                }
                let pos = (self.time() * 0.4 % 1.2) - 0.1;
                ui.rect(x, y + 70.0 * u, w, 6.0 * u, self.pal.outline_variant, 3.0 * u);
                ui.rect(x + w * pos, y + 70.0 * u, w * 0.25, 6.0 * u, self.pal.primary, 3.0 * u);
                ui.text(
                    "determinate - wavy - indeterminate",
                    x,
                    y + 88.0 * u,
                    TextStyle {
                        px: 12.0 * u,
                        medium: false,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
            }
            ComponentId::LoadingIndicator => {
                for i in 0..5 {
                    let ph = (self.time() * 3.0 + i as f32 * 0.9).sin() * 0.5 + 0.5;
                    ui.circle(
                        x + 40.0 * u + i as f32 * 48.0 * u,
                        y + 32.0 * u,
                        (6.0 + 8.0 * ph) * u,
                        self.pal.primary,
                    );
                }
                ui.text(
                    "expressive loading morph",
                    x + 40.0 * u + 5.0 * 48.0 * u,
                    y + 22.0 * u,
                    TextStyle {
                        px: 13.0 * u,
                        medium: false,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
            }
            ComponentId::AssistChip
            | ComponentId::FilterChip
            | ComponentId::InputChip
            | ComponentId::SuggestionChip => {
                let idx = match id {
                    ComponentId::AssistChip => 0,
                    ComponentId::FilterChip => 1,
                    ComponentId::InputChip => 2,
                    _ => 3,
                };
                let labels = ["Assist", "Filter", "Input", "Suggest"];
                let sel = self.chips[idx];
                let tw = ui.text_width(labels[idx], 13.0 * u, true, 0.0);
                let bw = tw + 56.0 * u;
                if sel {
                    ui.rect(x, y, bw, 38.0 * u, self.pal.secondary_container, 10.0 * u);
                } else {
                    self.frame(ui, x, y, bw, 38.0 * u, 10.0 * u, 1.5 * u, self.pal.outline);
                }
                ui.text(
                    labels[idx],
                    x + 28.0 * u,
                    y + 9.0 * u,
                    TextStyle {
                        px: 13.0 * u,
                        medium: true,
                        color: if sel { self.pal.on_secondary_container } else { self.pal.text },
                        tracking_em: 0.0,
                    },
                );
                self.zone(Action::Chip(idx), Rect { x, y, w: bw, h: 38.0 * u });
            }
            ComponentId::Dialog => {
                ui.rect(x, y, w.min(380.0 * u), 200.0 * u, self.pal.card_hi, 20.0 * u);
                let dw = w.min(380.0 * u);
                ui.text(
                    "Delete project?",
                    x + 20.0 * u,
                    y + 16.0 * u,
                    TextStyle {
                        px: 17.0 * u,
                        medium: true,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
                ui.text(
                    "This action cannot be undone.",
                    x + 20.0 * u,
                    y + 48.0 * u,
                    TextStyle {
                        px: 13.0 * u,
                        medium: false,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
                self.button(ui, 31, x + dw - 220.0 * u, y + 140.0 * u, 100.0 * u, 40.0 * u, "Cancel", 3, u);
                self.button(ui, 32, x + dw - 110.0 * u, y + 140.0 * u, 90.0 * u, 40.0 * u, "Delete", 0, u);
            }
            ComponentId::BottomSheet => {
                ui.rect(x, y, w, 180.0 * u, self.pal.card_hi, 20.0 * u);
                ui.rect(x + w * 0.5 - 24.0 * u, y + 10.0 * u, 48.0 * u, 5.0 * u, self.pal.outline, 2.5 * u);
                for i in 0..3 {
                    ui.text(
                        ["Share", "Copy link", "Remove"][i],
                        x + 20.0 * u,
                        y + 40.0 * u + i as f32 * 44.0 * u,
                        TextStyle {
                            px: 14.0 * u,
                            medium: false,
                            color: self.pal.text,
                            tracking_em: 0.0,
                        },
                    );
                }
            }
            ComponentId::NavigationBar => {
                let labels = ["Home", "Chat", "Mail", "Files", "More"];
                let cw = w / 5.0;
                for i in 0..5 {
                    let cx = x + i as f32 * cw;
                    if self.nav == i {
                        ui.rect(cx + 8.0 * u, y, cw - 16.0 * u, 34.0 * u, self.pal.secondary_container, 17.0 * u);
                    }
                    ui.circle(cx + cw * 0.5, y + 17.0 * u, 7.0 * u, if self.nav == i { self.pal.on_secondary_container } else { self.pal.sub });
                    ui.text(
                        labels[i],
                        cx + cw * 0.5 - 18.0 * u,
                        y + 42.0 * u,
                        TextStyle {
                            px: 11.0 * u,
                            medium: true,
                            color: if self.nav == i { self.pal.text } else { self.pal.sub },
                            tracking_em: 0.0,
                        },
                    );
                    self.zone(Action::Nav(i), Rect { x: cx, y, w: cw, h: 64.0 * u });
                }
            }
            ComponentId::NavigationRail => {
                for i in 0..4 {
                    let cy = y + i as f32 * 48.0 * u;
                    if self.rail == i {
                        ui.rect(x, cy, 76.0 * u, 40.0 * u, self.pal.secondary_container, 20.0 * u);
                    }
                    ui.circle(x + 38.0 * u, cy + 20.0 * u, 7.0 * u, if self.rail == i { self.pal.on_secondary_container } else { self.pal.sub });
                    ui.text(
                        ["A", "B", "C", "D"][i],
                        x + 92.0 * u,
                        cy + 8.0 * u,
                        TextStyle {
                            px: 13.0 * u,
                            medium: false,
                            color: self.pal.sub,
                            tracking_em: 0.0,
                        },
                    );
                    self.zone(Action::Rail(i), Rect { x, y: cy, w: 76.0 * u, h: 44.0 * u });
                }
            }
            ComponentId::NavigationDrawer => {
                ui.rect(x, y, 280.0 * u, 210.0 * u, self.pal.card_hi, 14.0 * u);
                ui.text(
                    "Geek Mail",
                    x + 16.0 * u,
                    y + 12.0 * u,
                    TextStyle {
                        px: 15.0 * u,
                        medium: true,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
                for i in 0..4 {
                    let iy = y + 48.0 * u + i as f32 * 38.0 * u;
                    if self.nav == i {
                        ui.rect(x + 8.0 * u, iy, 264.0 * u, 34.0 * u, self.pal.secondary_container, 17.0 * u);
                    }
                    ui.text(
                        ["Inbox", "Sent", "Drafts", "Trash"][i],
                        x + 28.0 * u,
                        iy + 7.0 * u,
                        TextStyle {
                            px: 13.0 * u,
                            medium: false,
                            color: if self.nav == i { self.pal.on_secondary_container } else { self.pal.text },
                            tracking_em: 0.0,
                        },
                    );
                    self.zone(Action::Nav(i), Rect { x: x + 8.0 * u, y: iy, w: 264.0 * u, h: 34.0 * u });
                }
            }
            ComponentId::Scaffold => {
                ui.rect(x, y, w.min(300.0 * u), 230.0 * u, self.pal.card_hi, 14.0 * u);
                let sw = w.min(300.0 * u);
                ui.text(
                    "App bar",
                    x + 16.0 * u,
                    y + 12.0 * u,
                    TextStyle {
                        px: 15.0 * u,
                        medium: true,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
                ui.text(
                    "Body content lives here.",
                    x + 16.0 * u,
                    y + 44.0 * u,
                    TextStyle {
                        px: 12.0 * u,
                        medium: false,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
                ui.circle(x + sw - 44.0 * u, y + 150.0 * u, 22.0 * u, self.pal.primary_container);
                ui.rect(x, y + 196.0 * u, sw, 34.0 * u, self.pal.card, 0.0);
                for i in 0..3 {
                    ui.circle(x + sw * (0.2 + i as f32 * 0.3), y + 213.0 * u, 6.0 * u, self.pal.sub);
                }
            }
            ComponentId::TopAppBar => {
                ui.text(
                    "Large title",
                    x,
                    y,
                    TextStyle {
                        px: 26.0 * u,
                        medium: true,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
                ui.text(
                    "Subtitle line goes here",
                    x,
                    y + 36.0 * u,
                    TextStyle {
                        px: 13.0 * u,
                        medium: false,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
                ui.circle(x + w - 30.0 * u, y + 20.0 * u, 18.0 * u, self.pal.card_hi);
                ui.text(
                    "o",
                    x + w - 37.0 * u,
                    y + 8.0 * u,
                    TextStyle {
                        px: 18.0 * u,
                        medium: false,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
            }
            ComponentId::SearchBar => {
                ui.rect(x, y, w, 52.0 * u, self.pal.card_hi, 26.0 * u);
                ui.text(
                    "Search components",
                    x + 52.0 * u,
                    y + 15.0 * u,
                    TextStyle {
                        px: 14.0 * u,
                        medium: false,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
                ui.circle(x + 28.0 * u, y + 26.0 * u, 9.0 * u, self.pal.sub);
            }
            ComponentId::TextField => {
                ui.rect(x, y, w, 62.0 * u, self.pal.card_hi, 6.0 * u);
                ui.text(
                    "Label",
                    x + 14.0 * u,
                    y + 8.0 * u,
                    TextStyle {
                        px: 11.0 * u,
                        medium: false,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
                ui.text(
                    "Hello geek",
                    x + 14.0 * u,
                    y + 28.0 * u,
                    TextStyle {
                        px: 15.0 * u,
                        medium: false,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
                let oy = y + 78.0 * u;
                self.frame(ui, x, oy, w, 62.0 * u, 6.0 * u, 1.5 * u, self.pal.outline);
                ui.text(
                    "Outlined",
                    x + 14.0 * u,
                    oy + 8.0 * u,
                    TextStyle {
                        px: 11.0 * u,
                        medium: false,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
                ui.text(
                    "secret",
                    x + 14.0 * u,
                    oy + 28.0 * u,
                    TextStyle {
                        px: 15.0 * u,
                        medium: false,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
            }
            ComponentId::Menu => {
                ui.rect(x, y, 220.0 * u, 210.0 * u, self.pal.card_hi, 8.0 * u);
                for i in 0..5 {
                    let iy = y + 8.0 * u + i as f32 * 38.0 * u;
                    if self.menu_sel == i {
                        ui.rect(x + 6.0 * u, iy, 208.0 * u, 34.0 * u, self.pal.secondary_container, 8.0 * u);
                    }
                    ui.text(
                        ["Cut", "Copy", "Paste", "Share", "Delete"][i],
                        x + 22.0 * u,
                        iy + 7.0 * u,
                        TextStyle {
                            px: 13.0 * u,
                            medium: false,
                            color: if self.menu_sel == i { self.pal.on_secondary_container } else { self.pal.text },
                            tracking_em: 0.0,
                        },
                    );
                    self.zone(Action::MenuItem(i), Rect { x: x + 6.0 * u, y: iy, w: 208.0 * u, h: 34.0 * u });
                }
            }
            ComponentId::Carousel => {
                for i in 0..3 {
                    let big = self.carousel == i;
                    let cw = if big { 150.0 * u } else { 110.0 * u };
                    let cx = x + i as f32 * 130.0 * u;
                    ui.rect(
                        cx,
                        y + if big { 0.0 } else { 12.0 * u },
                        cw,
                        if big { 120.0 * u } else { 96.0 * u },
                        if big {
                            self.pal.primary_container
                        } else {
                            self.pal.tertiary_container
                        },
                        12.0 * u,
                    );
                    ui.text(
                        format!("Card {}", i + 1).as_str(),
                        cx + 12.0 * u,
                        y + if big { 12.0 * u } else { 24.0 * u },
                        TextStyle {
                            px: 12.0 * u,
                            medium: true,
                            color: if big { self.pal.on_primary_container } else { self.pal.text },
                            tracking_em: 0.0,
                        },
                    );
                }
                for i in 0..3 {
                    ui.circle(
                        x + 20.0 * u + i as f32 * 28.0 * u,
                        y + 150.0 * u,
                        if self.carousel == i { 6.0 * u } else { 4.0 * u },
                        if self.carousel == i { self.pal.primary } else { self.pal.outline },
                    );
                    self.zone(
                        Action::CarouselPage(i),
                        Rect { x: x + 8.0 * u + i as f32 * 28.0 * u, y: y + 138.0 * u, w: 24.0 * u, h: 24.0 * u },
                    );
                }
            }
            ComponentId::DatePicker => {
                ui.text(
                    "October 2026",
                    x,
                    y,
                    TextStyle {
                        px: 16.0 * u,
                        medium: true,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
                for d in 0..7 {
                    ui.text(
                        ["M", "T", "W", "T", "F", "S", "S"][d],
                        x + 8.0 * u + d as f32 * 52.0 * u,
                        y + 34.0 * u,
                        TextStyle {
                            px: 12.0 * u,
                            medium: true,
                            color: self.pal.sub,
                            tracking_em: 0.0,
                        },
                    );
                }
                for day in 0..30 {
                    let gx = x + dcol(day, u);
                    let gy = y + 62.0 * u + (day / 7) as f32 * 44.0 * u;
                    if day == 13 {
                        ui.circle(gx + 20.0 * u, gy + 17.0 * u, 17.0 * u, self.pal.primary);
                    }
                    let label = format!("{}", day + 1);
                    let tw = ui.text_width(&label, 13.0 * u, day == 13, 0.0);
                    ui.text(
                        &label,
                        gx + 20.0 * u - tw * 0.5,
                        gy + 7.0 * u,
                        TextStyle {
                            px: 13.0 * u,
                            medium: day == 13,
                            color: if day == 13 { self.pal.on_primary } else { self.pal.text },
                            tracking_em: 0.0,
                        },
                    );
                }
            }
            ComponentId::TimePicker => {
                ui.circle(x + 90.0 * u, y + 90.0 * u, 80.0 * u, self.pal.card_hi);
                for (n, dx, dy) in [("12", 0.0, -62.0), ("3", 62.0, 0.0), ("6", 0.0, 62.0), ("9", -62.0, 0.0)] {
                    let tw = ui.text_width(n, 14.0 * u, false, 0.0);
                    ui.text(
                        n,
                        x + 90.0 * u + dx * u - tw * 0.5,
                        y + 90.0 * u + dy * u - 9.0 * u,
                        TextStyle {
                            px: 14.0 * u,
                            medium: false,
                            color: self.pal.text,
                            tracking_em: 0.0,
                        },
                    );
                }
                ui.rect(x + 88.0 * u, y + 40.0 * u, 4.0 * u, 52.0 * u, self.pal.primary, 2.0 * u);
                ui.circle(x + 90.0 * u, y + 90.0 * u, 8.0 * u, self.pal.primary);
                ui.text(
                    "10:24",
                    x + 200.0 * u,
                    y + 76.0 * u,
                    TextStyle {
                        px: 26.0 * u,
                        medium: true,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
            }
            ComponentId::Tooltip => {
                ui.rect(x + 40.0 * u, y, 220.0 * u, 40.0 * u, self.pal.card_hi, 8.0 * u);
                ui.text(
                    "Rich tooltip text",
                    x + 56.0 * u,
                    y + 10.0 * u,
                    TextStyle {
                        px: 13.0 * u,
                        medium: false,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
                self.button(ui, 33, x + 40.0 * u, y + 56.0 * u, 130.0 * u, 42.0 * u, "Anchor", 1, u);
            }
            ComponentId::Snackbar => {
                ui.rect(x, y, w, 56.0 * u, self.pal.card_hi, 8.0 * u);
                ui.text(
                    "Message sent",
                    x + 18.0 * u,
                    y + 17.0 * u,
                    TextStyle {
                        px: 14.0 * u,
                        medium: false,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
                ui.text(
                    "Undo",
                    x + w - 70.0 * u,
                    y + 17.0 * u,
                    TextStyle {
                        px: 14.0 * u,
                        medium: true,
                        color: self.pal.primary,
                        tracking_em: 0.0,
                    },
                );
            }
            ComponentId::Badge => {
                ui.circle(x + 30.0 * u, y + 30.0 * u, 24.0 * u, self.pal.card_hi);
                ui.text(
                    "o",
                    x + 22.0 * u,
                    y + 14.0 * u,
                    TextStyle {
                        px: 22.0 * u,
                        medium: false,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
                ui.circle(x + 48.0 * u, y + 14.0 * u, 12.0 * u, self.pal.error);
                ui.text(
                    "3",
                    x + 43.0 * u,
                    y + 3.0 * u,
                    TextStyle {
                        px: 13.0 * u,
                        medium: true,
                        color: self.pal.on_error,
                        tracking_em: 0.0,
                    },
                );
                ui.text(
                    "Badge with count",
                    x + 70.0 * u,
                    y + 20.0 * u,
                    TextStyle {
                        px: 14.0 * u,
                        medium: false,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
            }
            ComponentId::ListItem => {
                for i in 0..3 {
                    let ly = y + i as f32 * 58.0 * u;
                    ui.circle(x + 24.0 * u, ly + 24.0 * u, 20.0 * u, self.pal.secondary_container);
                    ui.text(
                        ["Kernel log", "Nightly build", "Release keys"][i],
                        x + 56.0 * u,
                        ly + 4.0 * u,
                        TextStyle {
                            px: 14.0 * u,
                            medium: false,
                            color: self.pal.text,
                            tracking_em: 0.0,
                        },
                    );
                    ui.text(
                        ["trace", "green", "sealed"][i],
                        x + 56.0 * u,
                        ly + 26.0 * u,
                        TextStyle {
                            px: 12.0 * u,
                            medium: false,
                            color: self.pal.sub,
                            tracking_em: 0.0,
                        },
                    );
                    ui.text(
                        "12:00",
                        x + w - 60.0 * u,
                        ly + 14.0 * u,
                        TextStyle {
                            px: 12.0 * u,
                            medium: false,
                            color: self.pal.sub,
                            tracking_em: 0.0,
                        },
                    );
                }
            }
            ComponentId::Tabs => {
                let labels = ["Code", "Build", "Ship", "Docs"];
                let tw2 = w / 4.0;
                for i in 0..4 {
                    let tx = x + i as f32 * tw2;
                    let sel = self.tabs == i;
                    let ltw = ui.text_width(labels[i], 14.0 * u, true, 0.0);
                    ui.text(
                        labels[i],
                        tx + (tw2 - ltw) * 0.5,
                        y + 8.0 * u,
                        TextStyle {
                            px: 14.0 * u,
                            medium: true,
                            color: if sel { self.pal.primary } else { self.pal.sub },
                            tracking_em: 0.0,
                        },
                    );
                    if sel {
                        ui.rect(tx + 12.0 * u, y + 40.0 * u, tw2 - 24.0 * u, 3.0 * u, self.pal.primary, 1.5 * u);
                    }
                    self.zone(Action::Tab(i), Rect { x: tx, y, w: tw2, h: 48.0 * u });
                }
            }
            ComponentId::SegmentedButton => {
                let labels = ["On", "Off", "Auto"];
                let sw = w / 3.0;
                for i in 0..3 {
                    let sx = x + i as f32 * sw;
                    let sel = self.segmented == i;
                    if sel {
                        ui.rect(sx, y, sw, 44.0 * u, self.pal.secondary_container, 0.0);
                    } else {
                        self.frame(ui, sx, y, sw, 44.0 * u, 0.0, 1.5 * u, self.pal.outline);
                    }
                    let ltw = ui.text_width(labels[i], 14.0 * u, true, 0.0);
                    ui.text(
                        labels[i],
                        sx + (sw - ltw) * 0.5,
                        y + 12.0 * u,
                        TextStyle {
                            px: 14.0 * u,
                            medium: true,
                            color: if sel { self.pal.on_secondary_container } else { self.pal.text },
                            tracking_em: 0.0,
                        },
                    );
                    self.zone(Action::Segmented(i), Rect { x: sx, y, w: sw, h: 44.0 * u });
                }
            }
            ComponentId::Divider => {
                ui.rect(x, y + 12.0 * u, w, 2.0 * u, self.pal.outline_variant, 1.0 * u);
            }
            ComponentId::PullToRefresh => {
                ui.circle(x + w * 0.5, y + 24.0 * u, 16.0 * u, self.pal.card);
                self.frame(ui, x + w * 0.5 - 16.0 * u, y + 8.0 * u, 32.0 * u, 32.0 * u, 16.0 * u, 2.5 * u, self.pal.primary);
                ui.text(
                    "Pull to refresh",
                    x + w * 0.5 - 60.0 * u,
                    y + 46.0 * u,
                    TextStyle {
                        px: 12.0 * u,
                        medium: false,
                        color: self.pal.sub,
                        tracking_em: 0.0,
                    },
                );
            }
            ComponentId::SwipeToDismiss => {
                ui.rect(x, y, w, 56.0 * u, self.pal.error, 10.0 * u);
                ui.text(
                    "Delete",
                    x + w - 80.0 * u,
                    y + 17.0 * u,
                    TextStyle {
                        px: 14.0 * u,
                        medium: true,
                        color: self.pal.on_error,
                        tracking_em: 0.0,
                    },
                );
                ui.rect(x, y, w - 90.0 * u, 56.0 * u, self.pal.card_hi, 10.0 * u);
                ui.text(
                    "Swipe me",
                    x + 18.0 * u,
                    y + 17.0 * u,
                    TextStyle {
                        px: 14.0 * u,
                        medium: false,
                        color: self.pal.text,
                        tracking_em: 0.0,
                    },
                );
            }
            ComponentId::Toolbar => {
                ui.rect(x, y, w, 56.0 * u, self.pal.card_hi, 12.0 * u);
                for i in 0..4 {
                    ui.circle(x + 30.0 * u + i as f32 * 56.0 * u, y + 28.0 * u, 14.0 * u, self.pal.secondary_container);
                    self.zone(
                        Action::Button(40 + i as u32),
                        Rect { x: x + 16.0 * u + i as f32 * 56.0 * u, y: y + 14.0 * u, w: 28.0 * u, h: 28.0 * u },
                    );
                }
            }
        }
    }
}

fn dcol(day: usize, u: f32) -> f32 {
    (day % 7) as f32 * 52.0 * u
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
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats.first().copied().unwrap_or(TextureFormat::Bgra8Unorm),
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
        let mut renderer = UiRenderer::new(&device, config.format);
        renderer.set_screen(&queue, config.width as f32, config.height as f32);
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
                .set_screen(&ui.queue, ui.config.width as f32, ui.config.height as f32);
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
