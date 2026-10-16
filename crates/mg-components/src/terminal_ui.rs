//! Material 3 Expressive UI components for Terminal and Developer tools.
//!
//! Provides top app bar with session tabs, an extra-keys toolbar, Ghostty customization
//! studio sheet, and quick actions dialog.

use crate::context::{
    icon_c, r4, text_c, ts, with_alpha, Rect, UiContext, WidgetId, ZoneKind,
};
use crate::props::{ChipProps, SliderProps};
use mg_render::{
    M3_BODY_MEDIUM, M3_LABEL_LARGE, M3_LABEL_MEDIUM, M3_LABEL_SMALL, M3_TITLE_LARGE,
    M3_TITLE_SMALL,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalTabItem {
    pub title: String,
    pub is_selected: bool,
}

impl TerminalTabItem {
    pub fn new(title: impl Into<String>, is_selected: bool) -> Self {
        Self {
            title: title.into(),
            is_selected,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalTopBarAction {
    SelectTab(usize),
    CloseTab(usize),
    NewTab,
    Paste,
    GhosttyStudio,
    QuickActions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtraKeyAction {
    Esc,
    Tab,
    ToggleCtrl,
    ToggleAlt,
    Slash,
    Pipe,
    Minus,
    Tilde,
    ArrowLeft,
    ArrowDown,
    ArrowUp,
    ArrowRight,
    ToggleKeyboard,
}

#[derive(Clone, Debug)]
pub struct GhosttyStudioState {
    pub is_open: bool,
    pub theme_index: usize,
    pub cursor_shader_mode: usize,
    pub cursor_style: usize,
    pub post_effect_flags: u32,
    pub font_size_dp: f32,
    pub slider_props: SliderProps,
    pub theme_scroll_x: f32,
    pub shader_scroll_x: f32,
    pub style_scroll_x: f32,
}

impl Default for GhosttyStudioState {
    fn default() -> Self {
        Self {
            is_open: false,
            theme_index: 0,
            cursor_shader_mode: 1, // Smear
            cursor_style: 0,       // Block
            post_effect_flags: 0,
            font_size_dp: 14.0,
            slider_props: SliderProps::with_range(14.0, 10.0, 26.0),
            theme_scroll_x: 0.0,
            shader_scroll_x: 0.0,
            style_scroll_x: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuickAction {
    Copy,
    Paste,
    SelectAll,
    ClearScreen,
    ResetTerminal,
}

impl<'a> UiContext<'a> {
    /// Renders an official Material 3 Expressive Terminal Top App Bar with session tabs,
    /// a new-tab button, grid size pill badge, quick actions button, and Ghostty Studio button.
    pub fn terminal_top_bar(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tabs: &[TerminalTabItem],
        grid_cols: usize,
        grid_rows: usize,
        tab_scroll_x: &mut f32,
    ) -> Option<TerminalTopBarAction> {
        let u = self.u();
        let pal = self.state.pal.clone();
        let mut action = None;

        // Container surface with M3 elevation color
        self.renderer
            .rect(x, y, w, h, pal.surface_container, r4(0.0));
        // Divider line at bottom
        self.renderer
            .rect(x, y + h - 1.0 * u, w, 1.0 * u, pal.outline_variant, r4(0.0));

        // Right side action buttons
        // 1. Ghostty Studio button
        let studio_rect = Rect::new(x + w - 44.0 * u, y + (h - 36.0 * u) * 0.5, 36.0 * u, 36.0 * u);
        let sid = WidgetId::from_label("tb_studio");
        self.draw_state_layer(sid, studio_rect, r4(18.0 * u), pal.primary);
        self.draw_ripples(sid, studio_rect);
        icon_c(
            self.renderer,
            "tune",
            studio_rect.cx(),
            studio_rect.cy(),
            20.0 * u,
            pal.primary,
        );
        if self
            .register_zone(sid, studio_rect, pal.primary, ZoneKind::Tap { bounded: true }, false)
            .clicked
        {
            action = Some(TerminalTopBarAction::GhosttyStudio);
        }

        // 2. Paste / Quick Actions button
        let paste_rect = Rect::new(x + w - 84.0 * u, y + (h - 36.0 * u) * 0.5, 36.0 * u, 36.0 * u);
        let pid = WidgetId::from_label("tb_paste");
        self.draw_state_layer(pid, paste_rect, r4(18.0 * u), pal.primary);
        self.draw_ripples(pid, paste_rect);
        icon_c(
            self.renderer,
            "content_paste",
            paste_rect.cx(),
            paste_rect.cy(),
            20.0 * u,
            pal.primary,
        );
        if self
            .register_zone(pid, paste_rect, pal.primary, ZoneKind::Tap { bounded: true }, false)
            .clicked
        {
            action = Some(TerminalTopBarAction::Paste);
        }

        // 3. Grid size pill badge
        let badge_text = format!("{}x{}", grid_cols, grid_rows);
        let badge_w = 64.0 * u;
        let badge_rect = Rect::new(
            x + w - 156.0 * u,
            y + (h - 24.0 * u) * 0.5,
            badge_w,
            24.0 * u,
        );
        self.renderer.rect(
            badge_rect.x,
            badge_rect.y,
            badge_rect.w,
            badge_rect.h,
            pal.surface_container_highest,
            r4(12.0 * u),
        );
        text_c(
            self.renderer,
            &badge_text,
            badge_rect.cx(),
            badge_rect.y + 4.0 * u,
            ts(M3_LABEL_SMALL, pal.sub, u),
        );

        // Left side: Session Tabs & New Tab (+) button
        let max_tab_area = (w - 164.0 * u).max(60.0 * u);

        // Precompute tab widths and total length
        let mut tab_widths = Vec::with_capacity(tabs.len());
        let mut total_tabs_w = 44.0 * u;
        for tab in tabs {
            let style = ts(M3_LABEL_LARGE, pal.text, u);
            let tw = self
                .renderer
                .text_width(&tab.title, style.px, style.medium, style.tracking_em);
            let has_close = tabs.len() > 1;
            let tab_w = (tw + 24.0 * u + if has_close { 20.0 * u } else { 0.0 })
                .min(140.0 * u)
                .max(64.0 * u);
            tab_widths.push(tab_w);
            total_tabs_w += tab_w + 6.0 * u;
        }

        let min_scroll = (max_tab_area - total_tabs_w).min(0.0);
        let tab_row_rect = Rect::new(x, y, max_tab_area, h);
        let tab_row_id = WidgetId::from_label("tb_tabs_row");
        let row_resp = self.register_zone(
            tab_row_id,
            tab_row_rect,
            pal.primary,
            ZoneKind::SwipeHorizontalDrag,
            true,
        );
        if row_resp.dragged {
            *tab_scroll_x = (*tab_scroll_x + self.frame_delta.0).clamp(min_scroll, 0.0);
        }

        let mut curr_x = x + 8.0 * u + *tab_scroll_x;

        for (i, tab) in tabs.iter().enumerate() {
            let tab_w = tab_widths[i];
            let tab_rect = Rect::new(curr_x, y + (h - 34.0 * u) * 0.5, tab_w, 34.0 * u);
            let tid = WidgetId::from_label("tb_tab").with_index(i);

            if tab_rect.x + tab_rect.w > x && tab_rect.x < x + max_tab_area {
                let text_color = if tab.is_selected {
                    pal.on_secondary_container
                } else {
                    pal.sub
                };
                let style = ts(M3_LABEL_LARGE, text_color, u);

                if tab.is_selected {
                    self.renderer.rect(
                        tab_rect.x,
                        tab_rect.y,
                        tab_rect.w,
                        tab_rect.h,
                        pal.secondary_container,
                        r4(17.0 * u),
                    );
                }
                self.draw_state_layer(tid, tab_rect, r4(17.0 * u), pal.text);
                self.draw_ripples(tid, tab_rect);

                let text_x = tab_rect.x + 12.0 * u;
                self.renderer.text(&tab.title, text_x, tab_rect.y + 8.0 * u, style);

                let has_close = tabs.len() > 1;
                if has_close {
                    let close_x = tab_rect.x + tab_rect.w - 16.0 * u;
                    let close_y = tab_rect.cy();
                    icon_c(self.renderer, "close", close_x, close_y, 14.0 * u, text_color);
                    let close_rect = Rect::new(close_x - 10.0 * u, tab_rect.y, 22.0 * u, tab_rect.h);
                    let cid = WidgetId::from_label("tb_close").with_index(i);
                    if self
                        .register_zone(cid, close_rect, pal.text, ZoneKind::Tap { bounded: true }, true)
                        .clicked
                    {
                        action = Some(TerminalTopBarAction::CloseTab(i));
                    }
                }

                if self
                    .register_zone(tid, tab_rect, pal.text, ZoneKind::Tap { bounded: true }, true)
                    .clicked
                    && action.is_none()
                {
                    action = Some(TerminalTopBarAction::SelectTab(i));
                }
            }

            curr_x += tab_w + 6.0 * u;
        }

        // New tab (+) button
        let add_rect = Rect::new(curr_x, y + (h - 32.0 * u) * 0.5, 32.0 * u, 32.0 * u);
        if add_rect.x + add_rect.w > x && add_rect.x < x + max_tab_area {
            let aid = WidgetId::from_label("tb_add");
            self.draw_state_layer(aid, add_rect, r4(16.0 * u), pal.primary);
            self.draw_ripples(aid, add_rect);
            icon_c(self.renderer, "add", add_rect.cx(), add_rect.cy(), 18.0 * u, pal.primary);
            if self
                .register_zone(aid, add_rect, pal.primary, ZoneKind::Tap { bounded: true }, true)
                .clicked
            {
                action = Some(TerminalTopBarAction::NewTab);
            }
        }

        action
    }

    /// Renders an official Material 3 Expressive Extra Keys Bar at the bottom of the screen.
    pub fn extra_keys_bar(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        ctrl_armed: bool,
        alt_armed: bool,
    ) -> Option<ExtraKeyAction> {
        let u = self.u();
        let pal = self.state.pal.clone();
        let mut action = None;

        // Container surface with low elevation
        self.renderer
            .rect(x, y, w, h, pal.surface_container_low, r4(0.0));
        // Divider line at top
        self.renderer
            .rect(x, y, w, 1.0 * u, pal.outline_variant, r4(0.0));

        // 13 keys: ESC, TAB, CTRL, ALT, /, |, -, ~, LEFT, DOWN, UP, RIGHT, KEYBOARD
        struct KeyDef {
            label: &'static str,
            icon: Option<&'static str>,
            action: ExtraKeyAction,
            is_active: bool,
        }

        let keys = [
            KeyDef { label: "ESC", icon: None, action: ExtraKeyAction::Esc, is_active: false },
            KeyDef { label: "TAB", icon: None, action: ExtraKeyAction::Tab, is_active: false },
            KeyDef { label: "CTRL", icon: None, action: ExtraKeyAction::ToggleCtrl, is_active: ctrl_armed },
            KeyDef { label: "ALT", icon: None, action: ExtraKeyAction::ToggleAlt, is_active: alt_armed },
            KeyDef { label: "/", icon: None, action: ExtraKeyAction::Slash, is_active: false },
            KeyDef { label: "|", icon: None, action: ExtraKeyAction::Pipe, is_active: false },
            KeyDef { label: "-", icon: None, action: ExtraKeyAction::Minus, is_active: false },
            KeyDef { label: "~", icon: None, action: ExtraKeyAction::Tilde, is_active: false },
            KeyDef { label: "", icon: Some("arrow_back"), action: ExtraKeyAction::ArrowLeft, is_active: false },
            KeyDef { label: "", icon: Some("arrow_downward"), action: ExtraKeyAction::ArrowDown, is_active: false },
            KeyDef { label: "", icon: Some("arrow_upward"), action: ExtraKeyAction::ArrowUp, is_active: false },
            KeyDef { label: "", icon: Some("arrow_forward"), action: ExtraKeyAction::ArrowRight, is_active: false },
            KeyDef { label: "", icon: Some("keyboard"), action: ExtraKeyAction::ToggleKeyboard, is_active: false },
        ];

        let count = keys.len() as f32;
        let pad_h = 4.0 * u;
        let spacing = 3.0 * u;
        let avail_w = w - pad_h * 2.0 - spacing * (count - 1.0);
        let key_w = (avail_w / count).max(22.0 * u);
        let key_h = (h - 8.0 * u).max(28.0 * u);
        let key_y = y + 4.0 * u;

        for (i, k) in keys.iter().enumerate() {
            let kx = x + pad_h + i as f32 * (key_w + spacing);
            let k_rect = Rect::new(kx, key_y, key_w, key_h);
            let kid = WidgetId::from_label("ek_key").with_index(i);

            let (bg_col, text_col) = if k.is_active {
                (pal.primary, pal.on_primary)
            } else {
                (pal.surface_container_high, pal.text)
            };

            self.renderer
                .rect(k_rect.x, k_rect.y, k_rect.w, k_rect.h, bg_col, r4(8.0 * u));
            self.draw_state_layer(kid, k_rect, r4(8.0 * u), text_col);
            self.draw_ripples(kid, k_rect);

            if let Some(ic) = k.icon {
                icon_c(self.renderer, ic, k_rect.cx(), k_rect.cy(), 18.0 * u, text_col);
            } else {
                text_c(
                    self.renderer,
                    k.label,
                    k_rect.cx(),
                    k_rect.y + (key_h - 16.0 * u) * 0.5,
                    ts(M3_LABEL_MEDIUM, text_col, u),
                );
            }

            if self
                .register_zone(kid, k_rect, text_col, ZoneKind::Tap { bounded: true }, false)
                .clicked
            {
                action = Some(k.action);
            }
        }

        action
    }

    /// Renders an official Material 3 Expressive Ghostty Studio BottomSheet.
    pub fn ghostty_studio_sheet(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        state: &mut GhosttyStudioState,
    ) -> bool {
        if !state.is_open {
            return false;
        }

        let u = self.u();
        let pal = self.state.pal.clone();
        let mut state_changed = false;

        let sheet_h = (screen_h * 0.72).min(560.0 * u);
        let sheet_y = screen_h - sheet_h;

        // Scrim ONLY covers area above the sheet
        let scrim_rect = Rect::new(0.0, 0.0, screen_w, sheet_y);
        self.renderer
            .rect(0.0, 0.0, screen_w, sheet_y, with_alpha(pal.scrim, 0.54), r4(0.0));
        let scrim_id = WidgetId::from_label("studio_scrim");
        if self
            .register_zone(scrim_id, scrim_rect, pal.text, ZoneKind::Tap { bounded: true }, true)
            .clicked
        {
            state.is_open = false;
            return true;
        }

        // Sheet container and its child widgets are overlays
        self.in_overlay = true;

        let sheet_rect = Rect::new(0.0, sheet_y, screen_w, sheet_h);
        let sheet_id = WidgetId::from_label("studio_sheet");
        let _ = self.register_zone(sheet_id, sheet_rect, pal.text, ZoneKind::Tap { bounded: true }, true);

        self.renderer.rect(
            sheet_rect.x,
            sheet_rect.y,
            sheet_rect.w,
            sheet_rect.h,
            pal.surface_container_low,
            [28.0 * u, 28.0 * u, 0.0, 0.0],
        );

        // Drag handle
        let handle_w = 32.0 * u;
        let handle_h = 4.0 * u;
        self.renderer.rect(
            (screen_w - handle_w) * 0.5,
            sheet_y + 12.0 * u,
            handle_w,
            handle_h,
            pal.outline,
            r4(2.0 * u),
        );

        // Header
        self.renderer.text(
            "Ghostty Studio",
            20.0 * u,
            sheet_y + 26.0 * u,
            ts(M3_TITLE_LARGE, pal.text, u),
        );
        self.renderer.text(
            "Shader animations & themes",
            20.0 * u,
            sheet_y + 52.0 * u,
            ts(M3_BODY_MEDIUM, pal.sub, u),
        );

        // Close button
        let close_rect = Rect::new(screen_w - 48.0 * u, sheet_y + 24.0 * u, 36.0 * u, 36.0 * u);
        let cid = WidgetId::from_label("studio_close");
        self.draw_state_layer(cid, close_rect, r4(18.0 * u), pal.sub);
        self.draw_ripples(cid, close_rect);
        icon_c(self.renderer, "close", close_rect.cx(), close_rect.cy(), 20.0 * u, pal.sub);
        if self
            .register_zone(cid, close_rect, pal.sub, ZoneKind::Tap { bounded: true }, true)
            .clicked
        {
            state.is_open = false;
            self.in_overlay = false;
            return true;
        }

        let mut cy = sheet_y + 80.0 * u;
        let row_w = screen_w - 40.0 * u;

        // Section 1: Themes (horizontally scrollable chip row)
        self.renderer.text(
            "Terminal Theme",
            20.0 * u,
            cy,
            ts(M3_TITLE_SMALL, pal.primary, u),
        );
        cy += 24.0 * u;

        let themes = [
            "Ghostty",
            "Catppuccin",
            "TokyoNight",
            "Gruvbox",
            "Dracula",
            "RosePine",
            "Nord",
            "Cyberpunk",
        ];
        let mut theme_widths = Vec::with_capacity(themes.len());
        let mut total_themes_w = 0.0f32;
        for name in &themes {
            let tw = self.renderer.text_width(name, 14.0 * u, true, 0.01);
            let cw = (tw + 32.0 * u).max(64.0 * u);
            theme_widths.push(cw);
            total_themes_w += cw + 8.0 * u;
        }
        let min_theme_scroll = (row_w - total_themes_w).min(0.0);
        let theme_row_rect = Rect::new(20.0 * u, cy, row_w, 36.0 * u);
        let theme_row_id = WidgetId::from_label("st_theme_row");
        let trow_resp = self.register_zone(
            theme_row_id,
            theme_row_rect,
            pal.primary,
            ZoneKind::SwipeHorizontalDrag,
            true,
        );
        if trow_resp.dragged {
            state.theme_scroll_x = (state.theme_scroll_x + self.frame_delta.0).clamp(min_theme_scroll, 0.0);
        }

        self.renderer.push_clip([20.0 * u, cy - 2.0 * u, row_w, 40.0 * u]);
        let mut tx = 20.0 * u + state.theme_scroll_x;
        for (i, name) in themes.into_iter().enumerate() {
            let cw = theme_widths[i];
            if tx + cw > 20.0 * u && tx < screen_w - 20.0 * u {
                let mut chip_props = ChipProps::new(name, state.theme_index == i);
                let tid = WidgetId::from_label("st_theme").with_index(i);
                let (_, resp, _) = self.chip(tid, tx, cy, &mut chip_props, false);
                if resp.clicked {
                    state.theme_index = i;
                    state_changed = true;
                }
            }
            tx += cw + 8.0 * u;
        }
        self.renderer.pop_clip();
        cy += 44.0 * u;

        // Section 2: Cursor Shader (horizontally scrollable chip row)
        self.renderer.text(
            "Cursor Animation Shader",
            20.0 * u,
            cy,
            ts(M3_TITLE_SMALL, pal.primary, u),
        );
        cy += 24.0 * u;

        let shaders = ["Off", "Smear", "Blaze", "Ripple", "Pulse"];
        let mut shader_widths = Vec::with_capacity(shaders.len());
        let mut total_shaders_w = 0.0f32;
        for name in &shaders {
            let tw = self.renderer.text_width(name, 14.0 * u, true, 0.01);
            let cw = (tw + 32.0 * u).max(60.0 * u);
            shader_widths.push(cw);
            total_shaders_w += cw + 8.0 * u;
        }
        let min_shader_scroll = (row_w - total_shaders_w).min(0.0);
        let shader_row_rect = Rect::new(20.0 * u, cy, row_w, 36.0 * u);
        let shader_row_id = WidgetId::from_label("st_shader_row");
        let srow_resp = self.register_zone(
            shader_row_id,
            shader_row_rect,
            pal.primary,
            ZoneKind::SwipeHorizontalDrag,
            true,
        );
        if srow_resp.dragged {
            state.shader_scroll_x = (state.shader_scroll_x + self.frame_delta.0).clamp(min_shader_scroll, 0.0);
        }

        self.renderer.push_clip([20.0 * u, cy - 2.0 * u, row_w, 40.0 * u]);
        let mut sx = 20.0 * u + state.shader_scroll_x;
        for (i, name) in shaders.into_iter().enumerate() {
            let cw = shader_widths[i];
            if sx + cw > 20.0 * u && sx < screen_w - 20.0 * u {
                let mut chip_props = ChipProps::new(name, state.cursor_shader_mode == i);
                let sid = WidgetId::from_label("st_shader").with_index(i);
                let (_, resp, _) = self.chip(sid, sx, cy, &mut chip_props, false);
                if resp.clicked {
                    state.cursor_shader_mode = i;
                    state_changed = true;
                }
            }
            sx += cw + 8.0 * u;
        }
        self.renderer.pop_clip();
        cy += 44.0 * u;

        // Section 3: Cursor Style (horizontally scrollable chip row)
        self.renderer.text(
            "Cursor Style",
            20.0 * u,
            cy,
            ts(M3_TITLE_SMALL, pal.primary, u),
        );
        cy += 24.0 * u;

        let styles = ["Block", "Bar", "Underline"];
        let mut style_widths = Vec::with_capacity(styles.len());
        let mut total_styles_w = 0.0f32;
        for name in &styles {
            let tw = self.renderer.text_width(name, 14.0 * u, true, 0.01);
            let cw = (tw + 32.0 * u).max(64.0 * u);
            style_widths.push(cw);
            total_styles_w += cw + 8.0 * u;
        }
        let min_style_scroll = (row_w - total_styles_w).min(0.0);
        let style_row_rect = Rect::new(20.0 * u, cy, row_w, 36.0 * u);
        let style_row_id = WidgetId::from_label("st_style_row");
        let styrow_resp = self.register_zone(
            style_row_id,
            style_row_rect,
            pal.primary,
            ZoneKind::SwipeHorizontalDrag,
            true,
        );
        if styrow_resp.dragged {
            state.style_scroll_x = (state.style_scroll_x + self.frame_delta.0).clamp(min_style_scroll, 0.0);
        }

        self.renderer.push_clip([20.0 * u, cy - 2.0 * u, row_w, 40.0 * u]);
        let mut stx = 20.0 * u + state.style_scroll_x;
        for (i, name) in styles.into_iter().enumerate() {
            let cw = style_widths[i];
            if stx + cw > 20.0 * u && stx < screen_w - 20.0 * u {
                let mut chip_props = ChipProps::new(name, state.cursor_style == i);
                let stid = WidgetId::from_label("st_style").with_index(i);
                let (_, resp, _) = self.chip(stid, stx, cy, &mut chip_props, false);
                if resp.clicked {
                    state.cursor_style = i;
                    state_changed = true;
                }
            }
            stx += cw + 8.0 * u;
        }
        self.renderer.pop_clip();
        cy += 44.0 * u;

        // Section 4: Post Effects
        self.renderer.text(
            "Post-Processing Shaders",
            20.0 * u,
            cy,
            ts(M3_TITLE_SMALL, pal.primary, u),
        );
        cy += 24.0 * u;

        let effects = [
            ("CRT Scanlines + Vignette", 1u32),
            ("Glyph Neon Bloom", 2u32),
            ("Cyber Aurora Grid", 4u32),
        ];

        for (name, flag) in effects {
            let mut is_active = (state.post_effect_flags & flag) != 0;
            let swid = WidgetId::from_label("st_fx").with_index(flag as usize);
            let resp = self.switch(swid, 20.0 * u, cy + 10.0 * u, name, &mut is_active, false);
            if resp.clicked {
                state.post_effect_flags ^= flag;
                state_changed = true;
            }
            cy += 40.0 * u;
        }

        // Section 5: Font size slider
        cy += 8.0 * u;
        let size_str = format!("Font Size: {:.1} pt", state.font_size_dp);
        self.renderer
            .text(&size_str, 20.0 * u, cy, ts(M3_TITLE_SMALL, pal.primary, u));
        cy += 26.0 * u;

        state.slider_props.value = state.font_size_dp;
        let slid = WidgetId::from_label("st_fontsize");
        let resp = self.slider(
            slid,
            20.0 * u,
            cy + 18.0 * u,
            screen_w - 40.0 * u,
            &mut state.slider_props,
            true,
            None,
        );
        if resp.changed {
            state.font_size_dp = state.slider_props.value;
            state_changed = true;
        }

        self.in_overlay = false;
        state_changed
    }

    /// Renders an official Material 3 Expressive Quick Actions BottomSheet.
    pub fn quick_actions_sheet(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        is_open: &mut bool,
    ) -> Option<QuickAction> {
        if !*is_open {
            return None;
        }

        let u = self.u();
        let pal = self.state.pal.clone();
        let mut action = None;

        let sheet_h = 320.0 * u;
        let sheet_y = screen_h - sheet_h;

        // Scrim ONLY covers area above the sheet
        let scrim_rect = Rect::new(0.0, 0.0, screen_w, sheet_y);
        self.renderer
            .rect(0.0, 0.0, screen_w, sheet_y, with_alpha(pal.scrim, 0.54), r4(0.0));
        let scrim_id = WidgetId::from_label("qa_scrim");
        if self
            .register_zone(scrim_id, scrim_rect, pal.text, ZoneKind::Tap { bounded: true }, true)
            .clicked
        {
            *is_open = false;
            return None;
        }

        self.in_overlay = true;

        // Sheet container
        let sheet_rect = Rect::new(0.0, sheet_y, screen_w, sheet_h);
        let sheet_id = WidgetId::from_label("qa_sheet");
        let _ = self.register_zone(sheet_id, sheet_rect, pal.text, ZoneKind::Tap { bounded: true }, true);

        self.renderer.rect(
            sheet_rect.x,
            sheet_rect.y,
            sheet_rect.w,
            sheet_rect.h,
            pal.surface_container_low,
            [28.0 * u, 28.0 * u, 0.0, 0.0],
        );

        // Drag handle
        self.renderer.rect(
            (screen_w - 32.0 * u) * 0.5,
            sheet_y + 12.0 * u,
            32.0 * u,
            4.0 * u,
            pal.outline,
            r4(2.0 * u),
        );

        // Title
        self.renderer.text(
            "Quick Actions",
            20.0 * u,
            sheet_y + 26.0 * u,
            ts(M3_TITLE_LARGE, pal.text, u),
        );

        let items = [
            ("content_copy", "Copy Selected Text", QuickAction::Copy),
            ("content_paste", "Paste From Clipboard", QuickAction::Paste),
            ("select_all", "Select All Characters", QuickAction::SelectAll),
            ("refresh", "Clear Screen Buffer", QuickAction::ClearScreen),
            ("history", "Reset Terminal Session", QuickAction::ResetTerminal),
        ];

        let mut cy = sheet_y + 68.0 * u;
        let item_h = 44.0 * u;

        for (i, (ic, label, act)) in items.into_iter().enumerate() {
            let row_rect = Rect::new(12.0 * u, cy, screen_w - 24.0 * u, item_h);
            let rid = WidgetId::from_label("qa_item").with_index(i);

            self.draw_state_layer(rid, row_rect, r4(12.0 * u), pal.text);
            self.draw_ripples(rid, row_rect);

            icon_c(self.renderer, ic, row_rect.x + 20.0 * u, row_rect.cy(), 22.0 * u, pal.primary);
            self.renderer.text(
                label,
                row_rect.x + 48.0 * u,
                row_rect.y + 12.0 * u,
                ts(M3_BODY_MEDIUM, pal.text, u),
            );

            if self
                .register_zone(rid, row_rect, pal.text, ZoneKind::Tap { bounded: true }, false)
                .clicked
            {
                action = Some(act);
                *is_open = false;
            }

            cy += item_h + 4.0 * u;
        }

        self.in_overlay = false;
        action
    }
}
