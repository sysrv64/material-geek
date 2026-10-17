use crate::context::{
    argb_hex, icon_c, lerp, loading_morph_state, mix, r4, text_c, ts, with_alpha, Rect,
    SoftKeyboardMode, UiContext, WidgetId, ZoneKind,
};
use crate::props::{
    ButtonProps, CheckState, ChipProps, DatePickerState, FabSize, IconButtonProps,
    IconButtonVariant, ProgressProps, PullRefreshState, Size, SliderProps, SplitButtonProps,
    SwipeDismissItem, TextFieldProps, TimePickerState, Variant,
};
use crate::ComponentId;
use mg_motion::{Tempo, Track};
use mg_render::{
    MaterialShape, M3_BODY_LARGE, M3_BODY_MEDIUM, M3_BODY_SMALL, M3_HEADLINE_SMALL, M3_LABEL_LARGE,
    M3_LABEL_MEDIUM, M3_LABEL_SMALL, M3_TITLE_LARGE, M3_TITLE_MEDIUM, M3_TITLE_SMALL,
};

pub const STATUS_BAR_INSET_DP: f32 = 38.0;
pub const APP_BAR_HEIGHT_DP: f32 = 64.0;

/// Width of the navigation pane of the adaptive list-detail layout, in dp.
pub const NAV_PANE_WIDTH_DP: f32 = 300.0;

/// Complete interactive state for the 41-component Material 3 Expressive Catalog built on the Material Geek UI Framework.
pub struct CatalogState {
    pub app_search_open: bool,
    pub app_search_field: TextFieldProps,
    pub toggle_btns: [bool; 3],
    pub icon_toggles: [bool; 4],
    pub ext_fab_expanded: bool,
    pub fab_menu_open: bool,
    pub fab_menu_last: Option<usize>,
    pub split_primary: SplitButtonProps,
    pub split_tonal: SplitButtonProps,
    pub group_sel: usize,
    pub std_group_sel: [bool; 4],
    pub card_selected: [bool; 3],
    pub checks: [CheckState; 3],
    pub switches: [bool; 3],
    pub radio: usize,
    pub slider_std: SliderProps,
    pub slider_wavy: SliderProps,
    pub range_slider: (f32, f32),
    pub chips: [ChipProps; 8],
    pub input_chip_visible: [bool; 2],
    pub dialog_open: bool,
    pub dialog_status: &'static str,
    pub sheet_expanded: bool,
    pub sheet_selected: usize,
    pub nav_sel: usize,
    pub rail_sel: usize,
    pub drawer_sel: usize,
    pub scaffold_tab: usize,
    pub scaffold_snack: bool,
    pub top_bar_expanded: bool,
    pub top_bar_starred: bool,
    pub search_bar_field: TextFieldProps,
    pub tf_email: TextFieldProps,
    pub tf_password: TextFieldProps,
    pub tf_username: TextFieldProps,
    pub menu_sel: usize,
    pub carousel_idx: usize,
    pub carousel_mode: usize,
    pub carousel_drag_accum: f32,
    pub date_picker: DatePickerState,
    pub time_picker: TimePickerState,
    pub tooltip_expanded: bool,
    pub snackbar_visible: bool,
    pub snackbar_undone: bool,
    pub badge_counts: [u32; 3],
    pub list_selected: [bool; 3],
    pub primary_tab: usize,
    pub secondary_tab: usize,
    pub segmented_sel: usize,
    pub pull_refresh: PullRefreshState,
    pub swipe_items: [SwipeDismissItem; 2],
    pub toolbar_sel: usize,
}

impl Default for CatalogState {
    fn default() -> Self {
        Self::new()
    }
}

impl CatalogState {
    pub fn new() -> Self {
        let mut app_search_field = TextFieldProps::outlined("Filter 41 M3E components...", "");
        app_search_field.tonal = true;
        app_search_field.leading_icon = Some("search".to_string());

        let mut search_bar_field =
            TextFieldProps::outlined("Search M3 Expressive components...", "");
        search_bar_field.tonal = true;
        search_bar_field.leading_icon = Some("search".to_string());
        search_bar_field.placeholder = "Type query (e.g. DatePicker, Slider)...".to_string();

        let mut tf_email =
            TextFieldProps::filled("Email address", "material.expressive@google.com");
        tf_email.leading_icon = Some("mail".to_string());
        tf_email.supporting = Some("Tap to edit with physical or on-screen keyboard".to_string());

        let mut tf_password = TextFieldProps::password("Password", "M3Expressive2026", true);
        tf_password.supporting = Some("Tap eye icon to toggle password visibility".to_string());

        let mut tf_username = TextFieldProps::outlined("Username", "geek_ui");
        tf_username.tonal = true;
        tf_username.leading_icon = Some("person".to_string());
        tf_username.prefix = Some("@".to_string());
        tf_username.max_length = Some(16);
        tf_username.supporting = Some("Expressive tonal field with character limit".to_string());

        let mut c0 = ChipProps::new("Add to calendar", false);
        c0.leading_icon = Some("calendar_today".to_string());
        let mut c1 = ChipProps::new("Share file", false);
        c1.leading_icon = Some("share".to_string());
        let mut c2 = ChipProps::new("Expressive", true);
        c2.leading_icon = Some("check".to_string());
        let mut c3 = ChipProps::new("Category", false);
        c3.trailing_icon = Some("expand_more".to_string());
        let mut c4 = ChipProps::new("alex@geek.dev", true);
        c4.leading_icon = Some("person".to_string());
        c4.trailing_icon = Some("close".to_string());
        let mut c5 = ChipProps::new("Attachment.pdf", true);
        c5.leading_icon = Some("attach_file".to_string());
        c5.trailing_icon = Some("close".to_string());
        let c6 = ChipProps::new("Set reminder", false);
        let mut c7 = ChipProps::new("Reply all", false);
        c7.leading_icon = Some("reply".to_string());

        let mut split_primary = SplitButtonProps::new(
            "Send mail",
            vec![
                "Send mail".to_string(),
                "Schedule send".to_string(),
                "Save draft".to_string(),
            ],
        );
        split_primary.leading_icon = Some("send".to_string());

        let mut split_tonal = SplitButtonProps::new(
            "Edit draft",
            vec![
                "Edit draft".to_string(),
                "Duplicate".to_string(),
                "Export PDF".to_string(),
            ],
        );
        split_tonal.leading_icon = Some("edit".to_string());
        split_tonal.tonal = true;

        let mut slider_std = SliderProps::new(0.62);
        slider_std.steps = 0;
        let slider_wavy = SliderProps::new(0.68);

        Self {
            app_search_open: false,
            app_search_field,
            toggle_btns: [true, false, false],
            icon_toggles: [true, false, true, false],
            ext_fab_expanded: true,
            fab_menu_open: true,
            fab_menu_last: None,
            split_primary,
            split_tonal,
            group_sel: 1,
            std_group_sel: [true, false, true, false],
            card_selected: [false, true, false],
            checks: [CheckState::On, CheckState::Off, CheckState::Indeterminate],
            switches: [true, false, true],
            radio: 0,
            slider_std,
            slider_wavy,
            range_slider: (0.22, 0.78),
            chips: [c0, c1, c2, c3, c4, c5, c6, c7],
            input_chip_visible: [true, true],
            dialog_open: true,
            dialog_status: "Ready",
            sheet_expanded: false,
            sheet_selected: 0,
            nav_sel: 0,
            rail_sel: 0,
            drawer_sel: 0,
            scaffold_tab: 0,
            scaffold_snack: true,
            top_bar_expanded: true,
            top_bar_starred: false,
            search_bar_field,
            tf_email,
            tf_password,
            tf_username,
            menu_sel: 1,
            carousel_idx: 0,
            carousel_mode: 0,
            carousel_drag_accum: 0.0,
            date_picker: DatePickerState::new(2026, 10, 16),
            time_picker: TimePickerState::new(10, 25, true),
            tooltip_expanded: true,
            snackbar_visible: true,
            snackbar_undone: false,
            badge_counts: [1, 3, 99],
            list_selected: [false, true, false],
            primary_tab: 0,
            secondary_tab: 1,
            segmented_sel: 0,
            pull_refresh: PullRefreshState::default(),
            swipe_items: [
                SwipeDismissItem::new(
                    1,
                    "Release v2.0 UI Framework",
                    "Drag left to delete or right to archive",
                    "Now",
                    "mail",
                    MaterialShape::Arch,
                ),
                SwipeDismissItem::new(
                    2,
                    "Design tokens & WGSL shaders",
                    "Spring physics settle & Undo support",
                    "14:20",
                    "star",
                    MaterialShape::Sunny,
                ),
            ],
            toolbar_sel: 0,
        }
    }
}

impl<'a> UiContext<'a> {
    pub fn component_title(id: ComponentId) -> &'static str {
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
            ComponentId::Slider => "Slider, WavySlider & RangeSlider",
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
            ComponentId::TextField => "TextField, OutlinedTextField & SecureTextField",
            ComponentId::Menu => "DropdownMenuPopup (Expressive Grouped)",
            ComponentId::Carousel => "HorizontalMultiBrowse & CenteredHero Carousel",
            ComponentId::DatePicker => "DatePickerDialog (Calendar, Year & Input Modes)",
            ComponentId::TimePicker => "TimePickerDialog (Dial & Numeric Input Modes)",
            ComponentId::Tooltip => "PlainTooltip & RichTooltip",
            ComponentId::Snackbar => "Snackbar",
            ComponentId::Badge => "Badge",
            ComponentId::ListItem => "ListItem (Expressive Segmented List)",
            ComponentId::Tabs => "PrimaryTabRow & SecondaryTabRow",
            ComponentId::SegmentedButton => "SingleChoiceSegmentedButtonRow",
            ComponentId::Divider => "HorizontalDivider",
            ComponentId::PullToRefresh => "PullToRefreshBox",
            ComponentId::SwipeToDismiss => "SwipeToDismissBox (Interactive Drag & Undo)",
            ComponentId::Toolbar => "HorizontalFloatingToolbar & FAB",
        }
    }

    pub fn component_subtitle(id: ComponentId) -> &'static str {
        match id {
            ComponentId::LoadingIndicator => "7-shape spring morph + 35 RoundedPolygons",
            ComponentId::ProgressIndicator => "Linear/Circular + Sinusoidal Wavy Shader",
            ComponentId::Carousel => "28dp corners, swipe or tap, dynamic hero squish",
            ComponentId::ButtonGroup => "Spring width physics + corner morphing",
            ComponentId::SplitButton => "Asymmetric 20dp/4dp corners + popup menu",
            ComponentId::FabMenu => "Expressive spring menu items + auto-collapse",
            ComponentId::Slider => "16dp track, wavy track, range slider & value bubble",
            ComponentId::DatePicker => "Tap pencil for numeric MM/DD/YYYY input mode",
            ComponentId::TimePicker => "Drag clock dial or tap keyboard for numeric input",
            ComponentId::TextField => "Live focus, caret, keyboard & password visibility",
            ComponentId::SwipeToDismiss => "Drag left to delete, right to archive, or Undo",
            _ => "Material 3 Expressive",
        }
    }

    pub fn body_height(id: ComponentId, cat: &CatalogState, u: f32) -> f32 {
        match id {
            ComponentId::Button => 152.0 * u,
            ComponentId::IconButton => 104.0 * u,
            ComponentId::Fab => 84.0 * u,
            ComponentId::ExtendedFab => 64.0 * u,
            ComponentId::FabMenu => 272.0 * u,
            ComponentId::SplitButton => {
                if cat.split_primary.menu_open || cat.split_tonal.menu_open {
                    176.0 * u
                } else {
                    52.0 * u
                }
            }
            ComponentId::ButtonGroup => 100.0 * u,
            ComponentId::Card => 176.0 * u,
            ComponentId::Checkbox => 120.0 * u,
            ComponentId::Switch => 136.0 * u,
            ComponentId::RadioButton => 120.0 * u,
            ComponentId::Slider => 176.0 * u,
            ComponentId::ProgressIndicator => 172.0 * u,
            ComponentId::LoadingIndicator => 136.0 * u,
            ComponentId::AssistChip
            | ComponentId::FilterChip
            | ComponentId::InputChip
            | ComponentId::SuggestionChip => 44.0 * u,
            ComponentId::Dialog => 224.0 * u,
            ComponentId::BottomSheet => {
                if cat.sheet_expanded {
                    232.0 * u
                } else {
                    184.0 * u
                }
            }
            ComponentId::NavigationBar => 80.0 * u,
            ComponentId::NavigationRail => 232.0 * u,
            ComponentId::NavigationDrawer => 288.0 * u,
            ComponentId::Scaffold => 232.0 * u,
            ComponentId::TopAppBar => 104.0 * u,
            ComponentId::SearchBar => 104.0 * u,
            ComponentId::TextField => 228.0 * u,
            ComponentId::Menu => 216.0 * u,
            ComponentId::Carousel => 218.0 * u,
            ComponentId::DatePicker => 444.0 * u,
            ComponentId::TimePicker => 436.0 * u,
            ComponentId::Tooltip => 128.0 * u,
            ComponentId::Snackbar => 64.0 * u,
            ComponentId::Badge => 56.0 * u,
            ComponentId::ListItem => 224.0 * u,
            ComponentId::Tabs => 118.0 * u,
            ComponentId::SegmentedButton => 48.0 * u,
            ComponentId::Divider => 52.0 * u,
            ComponentId::PullToRefresh => 72.0 * u,
            ComponentId::SwipeToDismiss => 164.0 * u,
            ComponentId::Toolbar => 64.0 * u,
        }
    }

    /// Scroll slot offset (in px, at the given density) of component `slot`,
    /// i.e. the distance from the top of the catalog list to the card top.
    fn slot_offset(slot: usize, cat: &CatalogState, u: f32) -> f32 {
        let mut off = 0.0;
        for (i, id) in ComponentId::ALL.iter().enumerate() {
            if i == slot {
                break;
            }
            off += (24.0 + 32.0 + Self::body_height(*id, cat, u) + 20.0) * u;
        }
        off
    }

    /// Adaptive layout router: the phone-style single-column catalog stays
    /// untouched; on wide viewports the M3 adaptive list-detail layout is
    /// used (navigation pane + scrollable detail pane).
    pub fn draw_catalog(&mut self, cat: &mut CatalogState) {
        if self.state.is_paned_layout() {
            self.draw_catalog_paned(cat);
        } else {
            self.draw_catalog_column(cat);
        }
    }

    /// Compact single-column catalog (phone behavior, unchanged).
    fn draw_catalog_column(&mut self, cat: &mut CatalogState) {
        let u = self.u();
        let width = self.state.width;
        let pad = 16.0 * u;
        let card_x = 8.0 * u;
        let card_w = width - 16.0 * u;
        let search_extra = if cat.app_search_open { 64.0 * u } else { 0.0 };
        let top_total = (self.state.top_inset_dp + APP_BAR_HEIGHT_DP) * u + search_extra;
        self.draw_component_list(cat, card_x, card_w, pad, top_total);
        self.draw_catalog_app_bar(width, cat);
        self.finish_frame();
    }

    /// Adaptive list-detail catalog: fixed navigation pane on the left,
    /// scrollable detail pane with the full component card list on the right.
    fn draw_catalog_paned(&mut self, cat: &mut CatalogState) {
        let u = self.u();
        let width = self.state.width;
        let nav_w = NAV_PANE_WIDTH_DP * u;
        let gap = 12.0 * u;
        let card_x = nav_w + gap;
        let card_w = width - card_x - 16.0 * u;
        let pad = 16.0 * u;
        let search_extra = if cat.app_search_open { 64.0 * u } else { 0.0 };
        let top_total = (self.state.top_inset_dp + APP_BAR_HEIGHT_DP) * u + search_extra;
        self.draw_component_list(cat, card_x, card_w, pad, top_total);
        self.draw_nav_pane(cat, nav_w, top_total);
        self.draw_catalog_app_bar(width, cat);
        self.finish_frame();
    }

    /// Scrollable card list used by both adaptive layouts (detail pane or
    /// full width on compact form factors). Draws the list body only.
    fn draw_component_list(
        &mut self,
        cat: &mut CatalogState,
        card_x: f32,
        card_w: f32,
        pad: f32,
        top_total: f32,
    ) {
        let u = self.u();
        let mut y = top_total + 14.0 * u - self.state.scroll;
        let filter_q = cat.app_search_field.text.trim().to_ascii_lowercase();

        for id in ComponentId::ALL {
            let title = Self::component_title(id);
            let sub = Self::component_subtitle(id);
            if !filter_q.is_empty()
                && !title.to_ascii_lowercase().contains(&filter_q)
                && !sub.to_ascii_lowercase().contains(&filter_q)
            {
                continue;
            }
            let pal = self.state.pal.clone();
            self.renderer
                .text(title, pad, y, ts(M3_LABEL_LARGE, pal.primary, u));
            let title_w = self.renderer.text_width(title, 14.0 * u, true, 0.1 / 14.0);
            self.renderer.text(
                sub,
                pad + title_w + 10.0 * u,
                y + 1.0 * u,
                ts(M3_LABEL_MEDIUM, pal.sub, u),
            );
            let body_h = Self::body_height(id, cat, u);
            let card_y = y + 24.0 * u;
            self.renderer.rect_styled(
                card_x,
                card_y,
                card_w,
                body_h + 32.0 * u,
                pal.card,
                pal.outline_variant,
                1.0 * u,
                r4(16.0 * u),
                0.0,
            );
            self.draw_component_body(
                id,
                card_x + 16.0 * u,
                card_y + 16.0 * u,
                card_w - 32.0 * u,
                cat,
            );
            y = card_y + body_h + 32.0 * u + 20.0 * u;
        }
        self.state.content_h = y + self.state.scroll + 24.0 * u;
        self.state.scroll_by(0.0);
    }

    /// Left navigation pane of the adaptive layout: clickable jump rows for
    /// all 41 catalog components with pane-local scroll and hover feedback.
    fn draw_nav_pane(&mut self, cat: &mut CatalogState, nav_w: f32, top_total: f32) {
        let u = self.u();
        let height = self.state.height;
        let pal = self.state.pal.clone();
        let pane_bg = pal.card;
        let row_h = 44.0 * u;

        // Floor of the visible pane area.
        let pane_top = top_total + 10.0 * u;
        let pane_bottom = height - 10.0 * u;
        let pane_h = (pane_bottom - pane_top).max(0.0);
        self.renderer.rect(0.0, pane_top, nav_w, pane_h, pane_bg, r4(20.0 * u));

        let visible = (pane_h / row_h).floor();
        let rows = ComponentId::ALL.len() as f32;
        let max = (rows - visible).max(0.0) * row_h;
        self.state.nav_scroll = self.state.nav_scroll.clamp(0.0, max);
        let first = (self.state.nav_scroll / row_h).floor() as usize;
        let visible_rows = (visible + 2.0) as usize;

        let filter_q = cat.app_search_field.text.trim().to_ascii_lowercase();
        for (idx, id) in ComponentId::ALL.iter().enumerate() {
            if idx < first || idx >= first + visible_rows {
                continue;
            }
            if !filter_q.is_empty()
                && !Self::component_title(*id)
                    .to_ascii_lowercase()
                    .contains(&filter_q)
            {
                continue;
            }
            let title = Self::component_title(*id);
            let row_x = 10.0 * u;
            let row_w = nav_w - 20.0 * u;
            let row_y =
                pane_top + 6.0 * u + (idx as f32) * row_h - self.state.nav_scroll;
            if row_y < pane_top || row_y + row_h > pane_bottom {
                continue;
            }
            let nav_id = WidgetId::from_label("nav_row_index").with_index(idx);
            let is_down = self.is_pressed(nav_id);
            self.draw_state_layer(nav_id, Rect::new(row_x, row_y, row_w, row_h), r4(28.0 * u), pal.primary);
            self.renderer.push_clip([row_x, pane_top, row_w, pane_h]);
            self.renderer.text(
                title,
                row_x + 18.0 * u,
                row_y + (row_h - 14.0 * u) / 2.0,
                ts(
                    M3_LABEL_LARGE,
                    if is_down { pal.primary } else { pal.text },
                    u,
                ),
            );
            self.renderer.pop_clip();
            let slot_off = Self::slot_offset(idx, cat, u);
            let target = (14.0 * u + slot_off - 8.0 * u).max(0.0);
            let respond = self.register_zone(
                nav_id,
                Rect::new(row_x, row_y, row_w, row_h),
                pal.primary,
                ZoneKind::Tap { bounded: true },
                false,
            );
            if respond.clicked {
                let max_scroll = (self.state.content_h - self.state.height).max(0.0);
                self.state.scroll = target.min(max_scroll);
            }
        }
    }

    fn draw_catalog_app_bar(&mut self, width: f32, cat: &mut CatalogState) {
        let u = self.u();
        let top_inset = self.state.top_inset_dp * u;
        let bar_h = APP_BAR_HEIGHT_DP * u;
        let extra_h = if cat.app_search_open { 64.0 * u } else { 0.0 };
        let total_h = top_inset + bar_h + extra_h;
        let pal = self.state.pal.clone();

        self.renderer
            .rect(0.0, 0.0, width, total_h, pal.bg, r4(0.0));
        let (sa, sb, mt, rot, _) = loading_morph_state(self.state.t, 0);
        self.renderer.m3_shape_morph(
            28.0 * u,
            top_inset + 32.0 * u,
            30.0 * u,
            sa,
            sb,
            mt,
            rot,
            pal.primary_container,
            pal.primary,
            1.2 * u,
        );
        self.renderer.text(
            "Material Geek UI",
            52.0 * u,
            top_inset + 18.0 * u,
            ts(M3_TITLE_LARGE, pal.text, u),
        );

        // Motion scheme pill toggle (Expressive vs Standard)
        let motion_lbl = if self.state.theme.expressive_motion {
            "M3E Spring"
        } else {
            "Standard"
        };
        let mut motion_chip = ChipProps::new(motion_lbl, self.state.theme.expressive_motion);
        if self
            .chip(
                WidgetId::from_label("app_bar_motion"),
                width - 224.0 * u,
                top_inset + 16.0 * u,
                &mut motion_chip,
                false,
            )
            .1
            .clicked
        {
            self.state.toggle_motion_scheme();
            cat.radio = if self.state.theme.expressive_motion {
                0
            } else {
                1
            };
        }

        // Search filter toggle button
        if self
            .icon_button(
                WidgetId::from_label("app_bar_search"),
                width - 96.0 * u,
                top_inset + 12.0 * u,
                if cat.app_search_open {
                    "close"
                } else {
                    "search"
                },
                &IconButtonProps::tonal(),
            )
            .clicked
        {
            cat.app_search_open = !cat.app_search_open;
            if cat.app_search_open {
                self.state.focus(
                    WidgetId::from_label("app_bar_search_field"),
                    SoftKeyboardMode::Qwerty,
                );
            } else {
                cat.app_search_field.clear();
                self.state.clear_focus();
            }
        }

        // Theme Dark / Light mode toggle button
        let theme_icon = if self.state.pal.is_dark {
            "light_mode"
        } else {
            "dark_mode"
        };
        if self
            .icon_button(
                WidgetId::from_label("app_bar_theme"),
                width - 48.0 * u,
                top_inset + 12.0 * u,
                theme_icon,
                &IconButtonProps::standard(),
            )
            .clicked
        {
            self.state.toggle_dark_mode();
        }

        if cat.app_search_open {
            let sf_rect = Rect::new(
                16.0 * u,
                top_inset + bar_h + 4.0 * u,
                width - 32.0 * u,
                48.0 * u,
            );
            self.text_field(
                WidgetId::from_label("app_bar_search_field"),
                sf_rect,
                &mut cat.app_search_field,
            );
        }

        self.renderer.rect(
            0.0,
            total_h - 1.0 * u,
            width,
            1.0 * u,
            pal.outline_variant,
            r4(0.5 * u),
        );
    }

    pub fn draw_component_body(
        &mut self,
        id: ComponentId,
        x: f32,
        y: f32,
        w: f32,
        cat: &mut CatalogState,
    ) {
        let u = self.u();
        let pal = self.state.pal.clone();
        match id {
            ComponentId::Button => {
                let mut bx = x;
                let variants = [
                    (1u32, "Filled", Variant::Filled),
                    (2, "Tonal", Variant::Tonal),
                    (3, "Outlined", Variant::Outlined),
                    (4, "Elevated", Variant::Elevated),
                    (5, "Text", Variant::Text),
                ];
                for (bid, lbl, var) in variants {
                    let (bw, _) = self.button(bid, bx, y, &ButtonProps::new(var, lbl));
                    bx += bw + if var == Variant::Elevated { 4.0 } else { 8.0 } * u;
                }

                // Row 2: M3 Expressive ToggleButtons
                let mut tx = x;
                let ty = y + 48.0 * u;
                let t_specs = [
                    ("Saved", "bookmark"),
                    ("Starred", "star"),
                    ("Liked", "favorite"),
                ];
                for (i, (lb, ic)) in t_specs.into_iter().enumerate() {
                    let (bw, _) = self.toggle_button(
                        WidgetId::from_label("toggle_btn").with_index(i),
                        tx,
                        ty,
                        lb,
                        ic,
                        &mut cat.toggle_btns[i],
                    );
                    tx += bw + 8.0 * u;
                }

                // Row 3: Interactive M3 Expressive Size Tiers (XS 32dp, S 40dp, M 56dp)
                let sy = y + 96.0 * u;
                self.button_rect(
                    WidgetId::from_label("btn_xs"),
                    Rect::new(x, sy + 12.0 * u, 76.0 * u, 32.0 * u),
                    &ButtonProps::tonal("XS 32dp").with_size(Size::Xs),
                );
                self.button_rect(
                    WidgetId::from_label("btn_s"),
                    Rect::new(x + 84.0 * u, sy + 8.0 * u, 84.0 * u, 40.0 * u),
                    &ButtonProps::outlined("S 40dp").with_size(Size::S),
                );
                let mw = (w - 176.0 * u).max(150.0 * u);
                self.button_rect(
                    WidgetId::from_label("btn_m"),
                    Rect::new(x + 176.0 * u, sy, mw, 56.0 * u),
                    &ButtonProps::filled("Expressive M (56dp)")
                        .with_icon("play_arrow")
                        .with_size(Size::M),
                );
            }
            ComponentId::IconButton => {
                let variants = [
                    IconButtonVariant::Standard,
                    IconButtonVariant::Filled,
                    IconButtonVariant::Tonal,
                    IconButtonVariant::Outlined,
                ];
                let icons = ["favorite", "add", "settings", "share"];
                for (i, v) in variants.into_iter().enumerate() {
                    let mut p = IconButtonProps::standard();
                    p.variant = v;
                    self.icon_button(
                        10 + i as u32,
                        x + i as f32 * 50.0 * u,
                        y + 4.0 * u,
                        icons[i],
                        &p,
                    );
                }
                self.icon_button_sized(
                    14u32,
                    Rect::new(x + 206.0 * u, y + 4.0 * u, 56.0 * u, 40.0 * u),
                    "mic",
                    &IconButtonProps::filled(),
                );
                self.icon_button_sized(
                    15u32,
                    Rect::new(x + 272.0 * u, y + 4.0 * u, 32.0 * u, 40.0 * u),
                    "more_vert",
                    &IconButtonProps::tonal(),
                );

                let ty = y + 56.0 * u;
                let t_icons = ["favorite", "bookmark", "notifications", "wifi"];
                for (i, ic) in t_icons.into_iter().enumerate() {
                    self.icon_toggle_button(
                        WidgetId::from_label("ic_toggle").with_index(i),
                        x + i as f32 * 52.0 * u,
                        ty,
                        ic,
                        &mut cat.icon_toggles[i],
                    );
                }
                self.renderer.text(
                    "Tap row 2 to morph toggle shapes",
                    x + 216.0 * u,
                    ty + 12.0 * u,
                    ts(M3_BODY_SMALL, pal.sub, u),
                );
            }
            ComponentId::Fab => {
                let mut fx = x;
                let (w1, _) = self.fab(
                    24u32,
                    fx,
                    y + 20.0 * u,
                    FabSize::Small,
                    "edit",
                    pal.secondary_container,
                    pal.on_secondary_container,
                );
                fx += w1 + 16.0 * u;
                let (w2, _) = self.fab(
                    20u32,
                    fx,
                    y + 12.0 * u,
                    FabSize::Medium,
                    "add",
                    pal.primary_container,
                    pal.on_primary_container,
                );
                fx += w2 + 16.0 * u;
                let (w3, _) = self.fab(
                    25u32,
                    fx,
                    y,
                    FabSize::Large,
                    "palette",
                    pal.tertiary_container,
                    pal.on_tertiary_container,
                );
                fx += w3 + 20.0 * u;
                let cx = fx + 32.0 * u;
                let cy = y + 40.0 * u;
                self.renderer.m3_shape(
                    cx,
                    cy,
                    64.0 * u,
                    MaterialShape::Cookie9Sided,
                    self.state.t * 0.35,
                    pal.primary,
                    [0.0; 4],
                    0.0,
                );
                icon_c(self.renderer, "star", cx, cy, 26.0 * u, pal.on_primary);
            }
            ComponentId::ExtendedFab => {
                let (fw, _) = self.extended_fab(
                    WidgetId::from_label("ext_fab_main"),
                    x,
                    y + 4.0 * u,
                    "New message",
                    "edit",
                    &mut cat.ext_fab_expanded,
                );
                let sx = x + fw + 14.0 * u;
                let mut always_exp = true;
                if sx + 140.0 * u <= x + w {
                    self.extended_fab(
                        WidgetId::from_label("ext_fab_nav"),
                        sx,
                        y + 4.0 * u,
                        "Navigate",
                        "explore",
                        &mut always_exp,
                    );
                }
            }
            ComponentId::FabMenu => {
                let open_t = self
                    .state
                    .spring_retarget(
                        WidgetId::from_label("fab_menu"),
                        0,
                        f32::from(cat.fab_menu_open),
                        Tempo::Default,
                        Track::Spatial,
                    )
                    .clamp(0.0, 1.0);
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
                    let ink = with_alpha(pal.on_primary_container, stagger);
                    let fill = with_alpha(pal.primary_container, stagger);
                    let style = ts(M3_LABEL_LARGE, ink, u);
                    let tw = self
                        .renderer
                        .text_width(lb, style.px, style.medium, style.tracking_em);
                    let full_iw = 18.0 * u + 22.0 * u + 10.0 * u + tw + 20.0 * u;
                    let iw = lerp(48.0 * u, full_iw, stagger);
                    let r = ih * 0.5;
                    let rect = Rect::new(x, iy, iw, ih);
                    let item_id = WidgetId::from_label("fab_menu_item").with_index(i);
                    self.renderer.rect(x, iy, iw, ih, fill, r4(r));
                    self.draw_state_layer(item_id, rect, r4(r), pal.on_primary_container);
                    self.draw_ripples(item_id, rect);
                    icon_c(self.renderer, ic, x + 26.0 * u, iy + ih * 0.5, 20.0 * u, ink);
                    if stagger > 0.05 {
                        self.renderer.push_clip([x, iy, (iw - 10.0 * u).max(0.0), ih]);
                        self.renderer
                            .text(lb, x + 46.0 * u, iy + ih * 0.5 - style.line_h * 0.5, style);
                        self.renderer.pop_clip();
                    }
                    if self
                        .register_zone(
                            item_id,
                            rect,
                            pal.on_primary_container,
                            ZoneKind::Tap { bounded: true },
                            false,
                        )
                        .clicked
                    {
                        cat.fab_menu_last = Some(i);
                        cat.fab_menu_open = false;
                    }
                }
                let fy = y + 214.0 * u;
                let fab_r = lerp(16.0 * u, 28.0 * u, open_t);
                let fab_fill = mix(pal.primary_container, pal.primary, open_t);
                let fab_ink = mix(pal.on_primary_container, pal.on_primary, open_t);
                if self
                    .fab_styled(
                        WidgetId::from_label("fab_menu_trigger"),
                        x,
                        fy,
                        56.0 * u,
                        fab_r,
                        fab_fill,
                        fab_ink,
                        if open_t > 0.5 { "close" } else { "add" },
                        24.0 * u,
                    )
                    .1
                    .clicked
                {
                    cat.fab_menu_open = !cat.fab_menu_open;
                }
                let status = if cat.fab_menu_open {
                    "Tap any item or X to collapse menu".to_string()
                } else if let Some(last) = cat.fab_menu_last {
                    format!("Selected: {} • Tap + to reopen", items[last].1)
                } else {
                    "Tap + to expand Expressive FAB menu".to_string()
                };
                self.renderer.text(
                    &status,
                    x + 72.0 * u,
                    fy + 18.0 * u,
                    ts(M3_BODY_SMALL, pal.sub, u),
                );
            }
            ComponentId::SplitButton => {
                let (w1, _, _) = self.split_button(
                    WidgetId::from_label("split_primary"),
                    x,
                    y + 6.0 * u,
                    &mut cat.split_primary,
                );
                let _ = self.split_button(
                    WidgetId::from_label("split_tonal"),
                    x + w1 + 14.0 * u,
                    y + 6.0 * u,
                    &mut cat.split_tonal,
                );
            }
            ComponentId::ButtonGroup => {
                let labels = ["Day", "Week", "Month", "Year"];
                let n = labels.len();
                let gap = 2.0 * u;
                let avail_w = w - gap * (n as f32 - 1.0);
                let h = 40.0 * u;
                let mut weights = [1.0f32; 4];
                for (i, wt) in weights.iter_mut().enumerate() {
                    let gid = WidgetId::from_label("conn_group").with_index(i);
                    let sel_t = self
                        .state
                        .spring_retarget(
                            gid,
                            0,
                            f32::from(cat.group_sel == i),
                            Tempo::Fast,
                            Track::Spatial,
                        )
                        .clamp(0.0, 1.2);
                    let press_t = self
                        .state
                        .spring_retarget(
                            gid,
                            1,
                            f32::from(self.is_pressed(gid)),
                            Tempo::Fast,
                            Track::Spatial,
                        )
                        .clamp(0.0, 1.2);
                    *wt += 0.14 * sel_t + 0.16 * press_t;
                }
                let weight_sum: f32 = weights.iter().sum();
                let mut gx = x;
                for (i, label) in labels.into_iter().enumerate() {
                    let gid = WidgetId::from_label("conn_group").with_index(i);
                    let sw = avail_w * (weights[i] / weight_sum);
                    let sel_t = self.state.spring(gid, 0, 0.0, true).clamp(0.0, 1.0);
                    let is_p = self.is_pressed(gid);
                    let fill = mix(pal.secondary_container, pal.primary, sel_t);
                    let ink = mix(pal.on_secondary_container, pal.on_primary, sel_t);
                    let base_inner = if is_p { 4.0 * u } else { 8.0 * u };
                    let inner = lerp(base_inner, 20.0 * u, sel_t);
                    let radii = if i == 0 {
                        [20.0 * u, inner, inner, 20.0 * u]
                    } else if i == n - 1 {
                        [inner, 20.0 * u, 20.0 * u, inner]
                    } else {
                        r4(inner)
                    };
                    let rect = Rect::new(gx, y, sw, h);
                    self.renderer
                        .rect(rect.x, rect.y, rect.w, rect.h, fill, radii);
                    self.draw_state_layer(gid, rect, radii, ink);
                    self.draw_ripples(gid, rect);
                    let style = ts(M3_LABEL_LARGE, ink, u);
                    let tw = self
                        .renderer
                        .text_width(label, style.px, style.medium, style.tracking_em);
                    let ic_w = 20.0 * u * sel_t;
                    let tx = gx + (sw - tw - ic_w) * 0.5;
                    if sel_t > 0.1 {
                        icon_c(
                            self.renderer,
                            "check",
                            tx + 7.0 * u,
                            y + h * 0.5,
                            16.0 * u * sel_t,
                            with_alpha(ink, sel_t),
                        );
                    }
                    self.renderer
                        .text(label, tx + ic_w, y + h * 0.5 - style.line_h * 0.5, style);
                    if self
                        .register_zone(gid, rect, ink, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.group_sel = i;
                    }
                    gx += sw + gap;
                }

                let icons = [
                    ("format_bold", "Bold"),
                    ("format_italic", "Italic"),
                    ("format_underlined", "Underline"),
                    ("link", "Link"),
                ];
                let ry = y + 52.0 * u;
                let mut sx = x;
                for (i, (ic, lb)) in icons.into_iter().enumerate() {
                    let sid = WidgetId::from_label("std_group").with_index(i);
                    let sel_t = self
                        .state
                        .spring_retarget(
                            sid,
                            0,
                            f32::from(cat.std_group_sel[i]),
                            Tempo::Fast,
                            Track::Spatial,
                        )
                        .clamp(0.0, 1.0);
                    let is_p = self.is_pressed(sid);
                    let fill = mix(pal.card_hi, pal.tertiary_container, sel_t);
                    let ink = mix(pal.text, pal.on_tertiary_container, sel_t);
                    let base_r = lerp(20.0 * u, 12.0 * u, sel_t);
                    let r = if is_p { 8.0 * u } else { base_r };
                    let style = ts(M3_LABEL_MEDIUM, ink, u);
                    let tw = self
                        .renderer
                        .text_width(lb, style.px, style.medium, style.tracking_em);
                    let bw = 14.0 * u + 20.0 * u + tw + 14.0 * u + 6.0 * u * sel_t;
                    let rect = Rect::new(sx, ry, bw, 40.0 * u);
                    self.renderer.rect(sx, ry, bw, 40.0 * u, fill, r4(r));
                    self.draw_state_layer(sid, rect, r4(r), ink);
                    self.draw_ripples(sid, rect);
                    icon_c(
                        self.renderer,
                        ic,
                        sx + 20.0 * u + 3.0 * u * sel_t,
                        ry + 20.0 * u,
                        18.0 * u,
                        ink,
                    );
                    self.renderer.text(
                        lb,
                        sx + 34.0 * u + 3.0 * u * sel_t,
                        ry + 20.0 * u - style.line_h * 0.5,
                        style,
                    );
                    if self
                        .register_zone(sid, rect, ink, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.std_group_sel[i] = !cat.std_group_sel[i];
                    }
                    sx += bw + 6.0 * u;
                }
            }
            ComponentId::Card => {
                let gap = 10.0 * u;
                let cw = (w - gap * 2.0) / 3.0;
                let ch = 172.0 * u;
                let specs = [
                    (
                        "Elevated",
                        "Shadow + SurfaceLow",
                        MaterialShape::Sunny,
                        pal.surface_low,
                        None,
                        true,
                        30u32,
                    ),
                    (
                        "Filled",
                        "SurfaceHighest fill",
                        MaterialShape::Clover4Leaf,
                        pal.card_highest,
                        None,
                        false,
                        31u32,
                    ),
                    (
                        "Outlined",
                        "1dp crisp border",
                        MaterialShape::Cookie9Sided,
                        pal.card,
                        Some(pal.outline),
                        false,
                        32u32,
                    ),
                ];
                for (i, (title, desc, shape, fill, border, elevated, bid)) in
                    specs.into_iter().enumerate()
                {
                    let cx = x + i as f32 * (cw + gap);
                    let cid = WidgetId::from(bid);
                    let sel = cat.card_selected[i];
                    if elevated {
                        let e = self.elev_dp(cid) * u;
                        self.renderer.rect(
                            cx,
                            y + e,
                            cw,
                            ch,
                            with_alpha(pal.bg, 0.7),
                            r4(16.0 * u),
                        );
                    }
                    self.renderer.rect_styled(
                        cx,
                        y,
                        cw,
                        ch,
                        fill,
                        if sel {
                            pal.primary
                        } else {
                            border.unwrap_or([0.0; 4])
                        },
                        if sel {
                            2.0 * u
                        } else if border.is_some() {
                            1.0 * u
                        } else {
                            0.0
                        },
                        r4(16.0 * u),
                        0.0,
                    );
                    self.renderer.push_clip([cx, y, cw, ch]);
                    self.renderer.rect(
                        cx,
                        y,
                        cw,
                        68.0 * u,
                        if i == 1 {
                            pal.tertiary_container
                        } else {
                            pal.primary_container
                        },
                        [16.0 * u, 16.0 * u, 6.0 * u, 6.0 * u],
                    );
                    self.renderer.m3_shape(
                        cx + cw - 28.0 * u,
                        y + 34.0 * u,
                        44.0 * u,
                        shape,
                        self.state.t * 0.25 + i as f32,
                        if i == 1 { pal.tertiary } else { pal.primary },
                        [0.0; 4],
                        0.0,
                    );
                    self.renderer.text(
                        title,
                        cx + 12.0 * u,
                        y + 78.0 * u,
                        ts(M3_TITLE_MEDIUM, pal.text, u),
                    );
                    self.renderer.text(
                        desc,
                        cx + 12.0 * u,
                        y + 102.0 * u,
                        ts(M3_BODY_SMALL, pal.sub, u),
                    );
                    self.renderer.pop_clip();
                    let btn_lbl = if sel { "Selected" } else { "Explore" };
                    let btn_props = if sel {
                        ButtonProps::filled(btn_lbl).with_icon("check")
                    } else {
                        ButtonProps::tonal(btn_lbl)
                    };
                    if self.button(bid, cx + 8.0 * u, y + 126.0 * u, &btn_props).1.clicked {
                        cat.card_selected[i] = !cat.card_selected[i];
                    }
                }
            }
            ComponentId::Checkbox => {
                let titles = ["Notifications", "Auto-sync", "Select all (Tri-state)"];
                for (i, title) in titles.into_iter().enumerate() {
                    self.checkbox(
                        WidgetId::from_label("chk").with_index(i),
                        x,
                        y + i as f32 * 40.0 * u,
                        title,
                        &mut cat.checks[i],
                        i == 2,
                    );
                }
            }
            ComponentId::Switch => {
                let specs = [
                    ("Adaptive notifications", false),
                    ("Quiet hours (with icons)", true),
                    ("Expressive haptics (with icons)", true),
                ];
                for (i, (label, with_ic)) in specs.into_iter().enumerate() {
                    self.switch(
                        WidgetId::from_label("sw").with_index(i),
                        x,
                        y + i as f32 * 44.0 * u,
                        label,
                        &mut cat.switches[i],
                        with_ic,
                    );
                }
            }
            ComponentId::RadioButton => {
                let labels = [
                    "Expressive motion (Default)",
                    "Standard motion",
                    "Reduced motion",
                ];
                for (i, label) in labels.into_iter().enumerate() {
                    if self
                        .radio_button(
                            WidgetId::from_label("rad").with_index(i),
                            x,
                            y + i as f32 * 40.0 * u,
                            label,
                            cat.radio == i,
                        )
                        .clicked
                    {
                        cat.radio = i;
                        let should_exp = i == 0;
                        if self.state.theme.expressive_motion != should_exp {
                            self.state.toggle_motion_scheme();
                        }
                    }
                }
            }
            ComponentId::Slider => {
                let pct1 = format!("{}%", (cat.slider_std.normalized() * 100.0).round() as i32);
                self.renderer.text(
                    "Standard Expressive Slider",
                    x,
                    y,
                    ts(M3_LABEL_MEDIUM, pal.sub, u),
                );
                let pct_style = ts(M3_LABEL_LARGE, pal.primary, u);
                let ptw =
                    self.renderer
                        .text_width(&pct1, pct_style.px, pct_style.medium, pct_style.tracking_em);
                self.renderer.text(&pct1, x + w - ptw, y, pct_style);
                self.slider(
                    WidgetId::from_label("slider_std"),
                    x,
                    y + 34.0 * u,
                    w,
                    &mut cat.slider_std,
                    false,
                    Some("volume_up"),
                );

                let pct2 = format!("{}%", (cat.slider_wavy.normalized() * 100.0).round() as i32);
                self.renderer.text(
                    "Wavy Expressive Slider (GPU Sinusoidal Wave)",
                    x,
                    y + 62.0 * u,
                    ts(M3_LABEL_MEDIUM, pal.sub, u),
                );
                let ptw2 =
                    self.renderer
                        .text_width(&pct2, pct_style.px, pct_style.medium, pct_style.tracking_em);
                self.renderer
                    .text(&pct2, x + w - ptw2, y + 62.0 * u, pct_style);
                self.slider(
                    WidgetId::from_label("slider_wavy"),
                    x,
                    y + 94.0 * u,
                    w,
                    &mut cat.slider_wavy,
                    true,
                    None,
                );

                let r_lbl = format!(
                    "RangeSlider: {}% – {}%",
                    (cat.range_slider.0 * 100.0).round() as i32,
                    (cat.range_slider.1 * 100.0).round() as i32
                );
                self.renderer.text(
                    &r_lbl,
                    x,
                    y + 120.0 * u,
                    ts(M3_LABEL_MEDIUM, pal.sub, u),
                );
                let (mut s_val, mut e_val) = cat.range_slider;
                self.range_slider(
                    WidgetId::from_label("slider_range"),
                    x,
                    y + 152.0 * u,
                    w,
                    &mut s_val,
                    &mut e_val,
                );
                cat.range_slider = (s_val, e_val);
            }
            ComponentId::ProgressIndicator => {
                let p = cat.slider_std.normalized();
                self.linear_progress(
                    x,
                    y + 8.0 * u,
                    w,
                    &ProgressProps {
                        progress: Some(p),
                        wavy: true,
                    },
                );
                self.linear_progress(
                    x,
                    y + 32.0 * u,
                    w,
                    &ProgressProps {
                        progress: None,
                        wavy: true,
                    },
                );
                self.linear_progress(
                    x,
                    y + 54.0 * u,
                    w,
                    &ProgressProps {
                        progress: Some(p),
                        wavy: false,
                    },
                );

                let cy = y + 116.0 * u;
                let r = 24.0 * u;
                let step_x = (w / 4.0).min(96.0 * u);
                let ind_start = (self.state.t * 180.0) % 360.0;
                let ind_sweep = 55.0 + 215.0 * ((self.state.t * 2.2).sin() * 0.5 + 0.5);
                let c_specs = [
                    (0, -90.0, p * 360.0, false, "Flat Det"),
                    (1, ind_start, ind_sweep, false, "Flat Indet"),
                    (2, -90.0, p * 360.0, true, "Wavy Det"),
                    (3, ind_start, ind_sweep, true, "Wavy Indet"),
                ];
                for (idx, s_deg, sw_deg, is_wavy, lbl) in c_specs {
                    let cx = x + step_x * (idx as f32 + 0.5);
                    self.circular_progress(cx, cy, r, 4.0 * u, s_deg, sw_deg, is_wavy);
                    text_c(
                        self.renderer,
                        lbl,
                        cx,
                        cy + 28.0 * u,
                        ts(M3_LABEL_SMALL, pal.sub, u),
                    );
                }
            }
            ComponentId::LoadingIndicator => {
                let cy = y + 30.0 * u;
                let (sa0, sb0, mt0, rot0, sc0) = loading_morph_state(self.state.t, 0);
                self.renderer.m3_shape_morph(
                    x + 30.0 * u,
                    cy,
                    38.0 * u * sc0,
                    sa0,
                    sb0,
                    mt0,
                    rot0,
                    pal.primary,
                    [0.0; 4],
                    0.0,
                );

                let (sa1, sb1, mt1, rot1, sc1) = loading_morph_state(self.state.t, 2);
                self.renderer
                    .circle(x + 98.0 * u, cy, 24.0 * u, pal.primary_container);
                self.renderer.m3_shape_morph(
                    x + 98.0 * u,
                    cy,
                    34.0 * u * sc1,
                    sa1,
                    sb1,
                    mt1,
                    rot1,
                    pal.on_primary_container,
                    [0.0; 4],
                    0.0,
                );

                let (sa2, sb2, mt2, rot2, sc2) = loading_morph_state(self.state.t, 4);
                self.renderer.m3_shape(
                    x + 170.0 * u,
                    cy,
                    54.0 * u,
                    MaterialShape::Circle,
                    0.0,
                    pal.tertiary_container,
                    [0.0; 4],
                    0.0,
                );
                self.renderer.m3_shape_morph(
                    x + 170.0 * u,
                    cy,
                    38.0 * u * sc2,
                    sa2,
                    sb2,
                    mt2,
                    rot2,
                    pal.on_tertiary_container,
                    [0.0; 4],
                    0.0,
                );

                self.renderer.text(
                    "7-Polygon Morph",
                    x + 216.0 * u,
                    y + 8.0 * u,
                    ts(M3_TITLE_SMALL, pal.text, u),
                );
                self.renderer.text(
                    "SoftBurst -> Cookie9 -> Pentagon -> Pill -> Sunny",
                    x + 216.0 * u,
                    y + 30.0 * u,
                    ts(M3_BODY_SMALL, pal.sub, u),
                );

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
                        pal.secondary_container
                    } else {
                        pal.primary_container
                    };
                    let stroke = if i % 2 == 0 {
                        pal.secondary
                    } else {
                        pal.primary
                    };
                    self.renderer
                        .m3_shape(sx, sy, 34.0 * u, sh, 0.0, fill, stroke, 1.2 * u);
                    text_c(
                        self.renderer,
                        name,
                        sx,
                        sy + 20.0 * u,
                        ts(M3_LABEL_SMALL, pal.sub, u),
                    );
                }
            }
            ComponentId::AssistChip => {
                let (w0, _, _) = self.chip(
                    WidgetId::from_label("chip_0"),
                    x,
                    y + 6.0 * u,
                    &mut cat.chips[0],
                    true,
                );
                let _ = self.chip(
                    WidgetId::from_label("chip_1"),
                    x + w0 + 8.0 * u,
                    y + 6.0 * u,
                    &mut cat.chips[1],
                    true,
                );
            }
            ComponentId::FilterChip => {
                cat.chips[2].leading_icon = if cat.chips[2].selected {
                    Some("check".to_string())
                } else {
                    None
                };
                cat.chips[3].leading_icon = if cat.chips[3].selected {
                    Some("check".to_string())
                } else {
                    None
                };
                let (w0, _, _) = self.chip(
                    WidgetId::from_label("chip_2"),
                    x,
                    y + 6.0 * u,
                    &mut cat.chips[2],
                    true,
                );
                let _ = self.chip(
                    WidgetId::from_label("chip_3"),
                    x + w0 + 8.0 * u,
                    y + 6.0 * u,
                    &mut cat.chips[3],
                    true,
                );
            }
            ComponentId::InputChip => {
                let mut cx = x;
                for idx in 0..2 {
                    if cat.input_chip_visible[idx] {
                        let (cw_chip, _, closed) = self.chip(
                            WidgetId::from_label("chip_in").with_index(idx),
                            cx,
                            y + 6.0 * u,
                            &mut cat.chips[4 + idx],
                            true,
                        );
                        if closed {
                            cat.input_chip_visible[idx] = false;
                        }
                        cx += cw_chip + 8.0 * u;
                    }
                }
                if !cat.input_chip_visible[0] || !cat.input_chip_visible[1] {
                    if self
                        .button(
                            WidgetId::from_label("restore_input_chips"),
                            cx,
                            y + 6.0 * u,
                            &ButtonProps::tonal("Restore chips").with_size(Size::Xs),
                        )
                        .1
                        .clicked
                    {
                        cat.input_chip_visible = [true, true];
                    }
                }
            }
            ComponentId::SuggestionChip => {
                let (w0, _, _) = self.chip(
                    WidgetId::from_label("chip_6"),
                    x,
                    y + 6.0 * u,
                    &mut cat.chips[6],
                    true,
                );
                let _ = self.chip(
                    WidgetId::from_label("chip_7"),
                    x + w0 + 8.0 * u,
                    y + 6.0 * u,
                    &mut cat.chips[7],
                    true,
                );
            }
            ComponentId::Dialog => {
                let dw = (340.0 * u).min(w);
                let dx = x + (w - dw) * 0.5;
                let dh = 216.0 * u;
                if !cat.dialog_open {
                    self.renderer.rect_styled(
                        dx,
                        y + 40.0 * u,
                        dw,
                        120.0 * u,
                        pal.surface_low,
                        pal.outline_variant,
                        1.0 * u,
                        r4(20.0 * u),
                        0.0,
                    );
                    text_c(
                        self.renderer,
                        &format!("Dialog closed ({})", cat.dialog_status),
                        dx + dw * 0.5,
                        y + 64.0 * u,
                        ts(M3_TITLE_MEDIUM, pal.text, u),
                    );
                    if self
                        .button(
                            WidgetId::from_label("reopen_dialog"),
                            dx + dw * 0.5 - 68.0 * u,
                            y + 102.0 * u,
                            &ButtonProps::filled("Reopen Dialog"),
                        )
                        .1
                        .clicked
                    {
                        cat.dialog_open = true;
                    }
                } else {
                    self.renderer.rect(
                        dx,
                        y + 4.0 * u,
                        dw,
                        dh,
                        with_alpha(pal.bg, 0.7),
                        r4(28.0 * u),
                    );
                    self.renderer.rect_styled(
                        dx,
                        y,
                        dw,
                        dh,
                        pal.card_hi,
                        pal.outline_variant,
                        1.0 * u,
                        r4(28.0 * u),
                        0.0,
                    );
                    self.renderer.m3_shape(
                        dx + dw * 0.5,
                        y + 34.0 * u,
                        40.0 * u,
                        MaterialShape::SoftBurst,
                        0.0,
                        pal.error_container,
                        [0.0; 4],
                        0.0,
                    );
                    icon_c(
                        self.renderer,
                        "delete",
                        dx + dw * 0.5,
                        y + 34.0 * u,
                        22.0 * u,
                        pal.on_error_container,
                    );
                    text_c(
                        self.renderer,
                        "Delete project?",
                        dx + dw * 0.5,
                        y + 62.0 * u,
                        ts(M3_HEADLINE_SMALL, pal.text, u),
                    );
                    text_c(
                        self.renderer,
                        "This permanent action removes all repositories",
                        dx + dw * 0.5,
                        y + 100.0 * u,
                        ts(M3_BODY_MEDIUM, pal.sub, u),
                    );
                    text_c(
                        self.renderer,
                        "and build artifacts from this workspace.",
                        dx + dw * 0.5,
                        y + 120.0 * u,
                        ts(M3_BODY_MEDIUM, pal.sub, u),
                    );
                    let st = ts(M3_LABEL_LARGE, pal.primary, u);
                    let w_c = 24.0 * u
                        + self
                            .renderer
                            .text_width("Cancel", st.px, st.medium, st.tracking_em)
                        + 24.0 * u;
                    let w_d = 40.0 * u
                        + self
                            .renderer
                            .text_width("Delete", st.px, st.medium, st.tracking_em);
                    let bx = dx + dw - w_c - w_d - 24.0 * u;
                    if self
                        .button(31u32, bx, y + 160.0 * u, &ButtonProps::text("Cancel"))
                        .1
                        .clicked
                    {
                        cat.dialog_status = "Cancelled";
                        cat.dialog_open = false;
                    }
                    if self
                        .button(
                            32u32,
                            bx + w_c + 8.0 * u,
                            y + 160.0 * u,
                            &ButtonProps::filled("Delete"),
                        )
                        .1
                        .clicked
                    {
                        cat.dialog_status = "Deleted";
                        cat.dialog_open = false;
                    }
                }
            }
            ComponentId::BottomSheet => {
                let sh = if cat.sheet_expanded {
                    228.0 * u
                } else {
                    180.0 * u
                };
                self.renderer.rect(
                    x,
                    y,
                    w,
                    sh,
                    pal.card_hi,
                    [28.0 * u, 28.0 * u, 12.0 * u, 12.0 * u],
                );
                let handle_id = WidgetId::from_label("sheet_handle");
                let handle_rect =
                    Rect::new(x + w * 0.5 - 36.0 * u, y + 4.0 * u, 72.0 * u, 24.0 * u);
                self.draw_state_layer(handle_id, handle_rect, r4(12.0 * u), pal.text);
                self.renderer.rect(
                    x + w * 0.5 - 16.0 * u,
                    y + 12.0 * u,
                    32.0 * u,
                    4.0 * u,
                    pal.outline,
                    r4(2.0 * u),
                );
                if self
                    .register_zone(
                        handle_id,
                        handle_rect,
                        pal.text,
                        ZoneKind::Tap { bounded: true },
                        false,
                    )
                    .clicked
                {
                    cat.sheet_expanded = !cat.sheet_expanded;
                }
                let items = [
                    ("share", "Share with collaborators"),
                    ("link", "Copy public link"),
                    ("archive", "Archive project"),
                    ("download", "Download offline bundle"),
                ];
                let count = if cat.sheet_expanded { 4 } else { 3 };
                for (i, (ic, lb)) in items.into_iter().take(count).enumerate() {
                    let iy = y + 32.0 * u + i as f32 * 46.0 * u;
                    let a = WidgetId::from(70 + i as u32);
                    let rect = Rect::new(x + 8.0 * u, iy, w - 16.0 * u, 44.0 * u);
                    let sel = cat.sheet_selected == i;
                    if sel {
                        self.renderer.rect(
                            rect.x,
                            rect.y,
                            rect.w,
                            rect.h,
                            pal.secondary_container,
                            r4(12.0 * u),
                        );
                    }
                    self.draw_state_layer(a, rect, r4(12.0 * u), pal.text);
                    self.draw_ripples(a, rect);
                    icon_c(self.renderer, ic, x + 28.0 * u, iy + 22.0 * u, 22.0 * u, pal.primary);
                    self.renderer.text(
                        lb,
                        x + 56.0 * u,
                        iy + 10.0 * u,
                        ts(M3_BODY_LARGE, pal.text, u),
                    );
                    if self
                        .register_zone(a, rect, pal.text, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.sheet_selected = i;
                    }
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
                self.renderer
                    .rect(x, y, w, 80.0 * u, pal.card_hi, r4(16.0 * u));
                let nav_pos = self.state.spring_retarget(
                    WidgetId::from_label("nav_bar"),
                    0,
                    cat.nav_sel as f32,
                    Tempo::Default,
                    Track::Spatial,
                );
                let pill_cx = x + (nav_pos + 0.5) * cw;
                self.renderer.rect(
                    pill_cx - 32.0 * u,
                    y + 12.0 * u,
                    64.0 * u,
                    32.0 * u,
                    pal.secondary_container,
                    r4(16.0 * u),
                );
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let cx0 = x + i as f32 * cw;
                    let sel = cat.nav_sel == i;
                    let ink = if sel {
                        pal.on_secondary_container
                    } else {
                        pal.sub
                    };
                    let rect = Rect::new(cx0, y, cw, 80.0 * u);
                    let nid = WidgetId::from_label("nav_item").with_index(i);
                    self.draw_state_layer(nid, rect, r4(16.0 * u), pal.text);
                    self.draw_ripples(nid, rect);
                    icon_c(self.renderer, ic, cx0 + cw * 0.5, y + 28.0 * u, 24.0 * u, ink);
                    let style = ts(M3_LABEL_MEDIUM, if sel { pal.text } else { pal.sub }, u);
                    text_c(self.renderer, lb, cx0 + cw * 0.5, y + 50.0 * u, style);
                    if self
                        .register_zone(nid, rect, ink, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.nav_sel = i;
                    }
                }
            }
            ComponentId::NavigationRail => {
                self.renderer
                    .rect(x, y, 88.0 * u, 232.0 * u, pal.surface_low, r4(16.0 * u));
                let items = [
                    ("home", "Home"),
                    ("search", "Search"),
                    ("bookmark", "Saved"),
                    ("person", "Profile"),
                ];
                let rail_pos = self.state.spring_retarget(
                    WidgetId::from_label("nav_rail"),
                    0,
                    cat.rail_sel as f32,
                    Tempo::Default,
                    Track::Spatial,
                );
                let pill_y = y + 8.0 * u + rail_pos * 54.0 * u + 4.0 * u;
                self.renderer.rect(
                    x + 16.0 * u,
                    pill_y,
                    56.0 * u,
                    32.0 * u,
                    pal.secondary_container,
                    r4(16.0 * u),
                );
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let iy = y + 8.0 * u + i as f32 * 54.0 * u;
                    let sel = cat.rail_sel == i;
                    let ink = if sel {
                        pal.on_secondary_container
                    } else {
                        pal.sub
                    };
                    let rect = Rect::new(x + 4.0 * u, iy, 80.0 * u, 52.0 * u);
                    let rid = WidgetId::from_label("rail_item").with_index(i);
                    self.draw_state_layer(rid, rect, r4(16.0 * u), pal.text);
                    self.draw_ripples(rid, rect);
                    icon_c(self.renderer, ic, x + 44.0 * u, iy + 20.0 * u, 22.0 * u, ink);
                    let style = ts(M3_LABEL_MEDIUM, if sel { pal.text } else { pal.sub }, u);
                    text_c(self.renderer, lb, x + 44.0 * u, iy + 36.0 * u, style);
                    if self
                        .register_zone(rid, rect, ink, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.rail_sel = i;
                    }
                }
                let px = x + 100.0 * u;
                let pw = (w - 100.0 * u).max(80.0 * u);
                self.renderer
                    .rect(px, y, pw, 232.0 * u, pal.card_hi, r4(16.0 * u));
                let dest_title = format!("Destination: {}", items[cat.rail_sel].1);
                self.renderer.text(
                    &dest_title,
                    px + 16.0 * u,
                    y + 18.0 * u,
                    ts(M3_TITLE_MEDIUM, pal.text, u),
                );
                self.renderer.text(
                    "Persistent side rail for tablet & desktop layouts.",
                    px + 16.0 * u,
                    y + 46.0 * u,
                    ts(M3_BODY_SMALL, pal.sub, u),
                );
            }
            ComponentId::NavigationDrawer => {
                let dw = (280.0 * u).min(w);
                self.renderer
                    .rect(x, y, dw, 288.0 * u, pal.card_hi, r4(16.0 * u));
                self.renderer.text(
                    "Expressive Mail",
                    x + 20.0 * u,
                    y + 16.0 * u,
                    ts(M3_TITLE_MEDIUM, pal.text, u),
                );
                let items = [
                    ("mail", "Inbox", "24"),
                    ("send", "Outbox", ""),
                    ("favorite", "Favorites", "5"),
                    ("delete", "Trash", ""),
                ];
                let drawer_pos = self.state.spring_retarget(
                    WidgetId::from_label("nav_drawer"),
                    0,
                    cat.drawer_sel as f32,
                    Tempo::Default,
                    Track::Spatial,
                );
                let pill_y = y + 52.0 * u + drawer_pos * 56.0 * u;
                self.renderer.rect(
                    x + 12.0 * u,
                    pill_y,
                    dw - 24.0 * u,
                    52.0 * u,
                    pal.secondary_container,
                    r4(26.0 * u),
                );
                for (i, (ic, lb, badge)) in items.into_iter().enumerate() {
                    let iy = y + 52.0 * u + i as f32 * 56.0 * u;
                    let sel = cat.drawer_sel == i;
                    let ink = if sel {
                        pal.on_secondary_container
                    } else {
                        pal.text
                    };
                    let rect = Rect::new(x + 12.0 * u, iy, dw - 24.0 * u, 52.0 * u);
                    let did = WidgetId::from_label("drawer_item").with_index(i);
                    self.draw_state_layer(did, rect, r4(26.0 * u), pal.text);
                    self.draw_ripples(did, rect);
                    icon_c(self.renderer, ic, x + 32.0 * u, iy + 26.0 * u, 22.0 * u, ink);
                    self.renderer
                        .text(lb, x + 56.0 * u, iy + 16.0 * u, ts(M3_LABEL_LARGE, ink, u));
                    if !badge.is_empty() {
                        let st = ts(M3_LABEL_MEDIUM, ink, u);
                        let btw = self
                            .renderer
                            .text_width(badge, st.px, st.medium, st.tracking_em);
                        self.renderer
                            .text(badge, x + dw - 28.0 * u - btw, iy + 18.0 * u, st);
                    }
                    if self
                        .register_zone(did, rect, ink, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.drawer_sel = i;
                    }
                }
            }
            ComponentId::Scaffold => {
                let sw = (340.0 * u).min(w);
                self.renderer.rect_styled(
                    x,
                    y,
                    sw,
                    232.0 * u,
                    pal.surface_low,
                    pal.outline_variant,
                    1.0 * u,
                    r4(16.0 * u),
                    0.0,
                );
                self.renderer.push_clip([x, y, sw, 232.0 * u]);
                self.renderer.rect(x, y, sw, 52.0 * u, pal.card_hi, r4(0.0));
                let tabs_names = ["Home Feed", "Search View", "User Profile"];
                self.renderer.text(
                    tabs_names[cat.scaffold_tab % 3],
                    x + 16.0 * u,
                    y + 14.0 * u,
                    ts(M3_TITLE_MEDIUM, pal.text, u),
                );
                icon_c(
                    self.renderer,
                    "more_vert",
                    x + sw - 24.0 * u,
                    y + 26.0 * u,
                    20.0 * u,
                    pal.sub,
                );
                self.renderer.rect(
                    x + 16.0 * u,
                    y + 66.0 * u,
                    sw - 32.0 * u,
                    40.0 * u,
                    pal.card_hi,
                    r4(10.0 * u),
                );
                self.renderer.text(
                    "TopAppBar + Content + FAB + NavigationBar",
                    x + 28.0 * u,
                    y + 78.0 * u,
                    ts(M3_BODY_SMALL, pal.sub, u),
                );
                if cat.scaffold_snack {
                    self.renderer.rect(
                        x + 16.0 * u,
                        y + 122.0 * u,
                        sw - 92.0 * u,
                        36.0 * u,
                        pal.inverse,
                        r4(8.0 * u),
                    );
                    self.renderer.text(
                        "Action triggered",
                        x + 28.0 * u,
                        y + 132.0 * u,
                        ts(M3_BODY_SMALL, pal.on_inverse, u),
                    );
                }
                if self
                    .fab_styled(
                        80u32,
                        x + sw - 64.0 * u,
                        y + 114.0 * u,
                        48.0 * u,
                        14.0 * u,
                        pal.primary_container,
                        pal.on_primary_container,
                        "add",
                        22.0 * u,
                    )
                    .1
                    .clicked
                {
                    cat.scaffold_snack = !cat.scaffold_snack;
                }
                self.renderer
                    .rect(x, y + 176.0 * u, sw, 56.0 * u, pal.card_hi, r4(0.0));
                let icons = ["home", "search", "person"];
                for (i, ic) in icons.into_iter().enumerate() {
                    let cx = x + sw * ((i as f32 + 0.5) / 3.0);
                    let sel = cat.scaffold_tab == i;
                    let s_id = WidgetId::from_label("scaf_nav").with_index(i);
                    let s_rect = Rect::new(cx - 32.0 * u, y + 178.0 * u, 64.0 * u, 50.0 * u);
                    if sel {
                        self.renderer.rect(
                            cx - 24.0 * u,
                            y + 188.0 * u,
                            48.0 * u,
                            28.0 * u,
                            pal.secondary_container,
                            r4(14.0 * u),
                        );
                    }
                    icon_c(
                        self.renderer,
                        ic,
                        cx,
                        y + 202.0 * u,
                        20.0 * u,
                        if sel {
                            pal.on_secondary_container
                        } else {
                            pal.sub
                        },
                    );
                    if self
                        .register_zone(
                            s_id,
                            s_rect,
                            pal.text,
                            ZoneKind::Tap { bounded: true },
                            false,
                        )
                        .clicked
                    {
                        cat.scaffold_tab = i;
                    }
                }
                self.renderer.pop_clip();
            }
            ComponentId::TopAppBar => {
                self.renderer
                    .rect(x, y, w, 100.0 * u, pal.card_hi, r4(16.0 * u));
                if self
                    .icon_button(
                        90u32,
                        x + 6.0 * u,
                        y + 8.0 * u,
                        "arrow_back",
                        &IconButtonProps::standard(),
                    )
                    .clicked
                {
                    cat.top_bar_expanded = !cat.top_bar_expanded;
                }
                if self
                    .icon_button(
                        91u32,
                        x + w - 92.0 * u,
                        y + 8.0 * u,
                        if cat.top_bar_starred {
                            "favorite"
                        } else {
                            "attach_file"
                        },
                        &IconButtonProps::tonal(),
                    )
                    .clicked
                {
                    cat.top_bar_starred = !cat.top_bar_starred;
                }
                self.icon_button(
                    92u32,
                    x + w - 46.0 * u,
                    y + 8.0 * u,
                    "more_vert",
                    &IconButtonProps::standard(),
                );
                self.renderer.text(
                    if cat.top_bar_expanded {
                        "Expressive Catalog"
                    } else {
                        "Collapsed TopAppBar"
                    },
                    x + 16.0 * u,
                    y + 50.0 * u,
                    ts(M3_HEADLINE_SMALL, pal.text, u),
                );
                self.renderer.text(
                    "MediumFlexibleTopAppBar • Tap arrows/icons",
                    x + 16.0 * u,
                    y + 78.0 * u,
                    ts(M3_LABEL_MEDIUM, pal.sub, u),
                );
            }
            ComponentId::SearchBar => {
                let sw = w;
                let s_rect = Rect::new(x, y, sw, 56.0 * u);
                self.text_field(
                    WidgetId::from_label("catalog_search_bar"),
                    s_rect,
                    &mut cat.search_bar_field,
                );
                let suggestions = ["DatePicker", "TextField", "SwipeToDismiss", "WavySlider"];
                let mut sx = x;
                let sy = y + 66.0 * u;
                for (i, sug) in suggestions.into_iter().enumerate() {
                    let mut chip =
                        ChipProps::new(sug, cat.search_bar_field.text.eq_ignore_ascii_case(sug));
                    let (cw_s, resp, _) = self.chip(
                        WidgetId::from_label("search_sug").with_index(i),
                        sx,
                        sy,
                        &mut chip,
                        false,
                    );
                    if resp.clicked {
                        cat.search_bar_field.text = sug.to_string();
                        cat.search_bar_field.cursor = sug.len();
                    }
                    sx += cw_s + 8.0 * u;
                }
            }
            ComponentId::TextField => {
                let r1 = Rect::new(x, y, w, 56.0 * u);
                self.text_field(WidgetId::from_label("tf_email"), r1, &mut cat.tf_email);

                let r2 = Rect::new(x, y + 82.0 * u, w, 56.0 * u);
                self.text_field(
                    WidgetId::from_label("tf_password"),
                    r2,
                    &mut cat.tf_password,
                );

                let r3 = Rect::new(x, y + 162.0 * u, w, 52.0 * u);
                self.text_field(
                    WidgetId::from_label("tf_username"),
                    r3,
                    &mut cat.tf_username,
                );
            }
            ComponentId::Menu => {
                let mw = (260.0 * u).min(w);
                let items = [
                    ("edit", "Rename file", "Ctrl+R"),
                    ("share", "Share link", "Ctrl+S"),
                    ("archive", "Move to archive", ""),
                    ("delete", "Delete permanently", "Del"),
                ];
                let mut iy = y;
                for (i, (ic, lb, shortcut)) in items.into_iter().enumerate() {
                    let sel = cat.menu_sel == i;
                    let fill = if sel {
                        pal.secondary_container
                    } else {
                        pal.card_hi
                    };
                    let ink = if sel {
                        pal.on_secondary_container
                    } else if i == 3 {
                        pal.error
                    } else {
                        pal.text
                    };
                    let radii = match i {
                        0 => [16.0 * u, 16.0 * u, 4.0 * u, 4.0 * u],
                        1 => r4(4.0 * u),
                        2 => [4.0 * u, 4.0 * u, 16.0 * u, 16.0 * u],
                        _ => r4(16.0 * u),
                    };
                    let rect = Rect::new(x, iy, mw, 48.0 * u);
                    let mid = WidgetId::from_label("menu_item").with_index(i);
                    self.renderer
                        .rect(rect.x, rect.y, rect.w, rect.h, fill, radii);
                    self.draw_state_layer(mid, rect, radii, ink);
                    self.draw_ripples(mid, rect);
                    icon_c(self.renderer, ic, x + 22.0 * u, iy + 24.0 * u, 20.0 * u, ink);
                    self.renderer
                        .text(lb, x + 46.0 * u, iy + 14.0 * u, ts(M3_BODY_LARGE, ink, u));
                    if sel {
                        icon_c(
                            self.renderer,
                            "check",
                            x + mw - 22.0 * u,
                            iy + 24.0 * u,
                            18.0 * u,
                            ink,
                        );
                    } else if !shortcut.is_empty() {
                        let st = ts(M3_LABEL_SMALL, pal.sub, u);
                        let stw = self
                            .renderer
                            .text_width(shortcut, st.px, st.medium, st.tracking_em);
                        self.renderer
                            .text(shortcut, x + mw - 16.0 * u - stw, iy + 16.0 * u, st);
                    }
                    if self
                        .register_zone(mid, rect, ink, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.menu_sel = i;
                    }
                    iy += if i == 2 { 56.0 * u } else { 50.0 * u };
                }
            }
            ComponentId::Carousel => {
                let modes = ["Multi-browse", "Centered hero", "Uncontained"];
                let mut mx = x;
                for (m, label) in modes.into_iter().enumerate() {
                    let sel = cat.carousel_mode == m;
                    let fill = if sel { pal.primary } else { pal.card_hi };
                    let ink = if sel { pal.on_primary } else { pal.text };
                    let st = ts(M3_LABEL_MEDIUM, ink, u);
                    let tw = self
                        .renderer
                        .text_width(label, st.px, st.medium, st.tracking_em);
                    let mw = tw + 24.0 * u;
                    let m_rect = Rect::new(mx, y, mw, 28.0 * u);
                    let mid = WidgetId::from_label("car_mode").with_index(m);
                    self.renderer.rect(mx, y, mw, 28.0 * u, fill, r4(14.0 * u));
                    self.draw_state_layer(mid, m_rect, r4(14.0 * u), ink);
                    self.draw_ripples(mid, m_rect);
                    self.renderer.text(label, mx + 12.0 * u, y + 6.0 * u, st);
                    if self
                        .register_zone(mid, m_rect, ink, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.carousel_mode = m;
                        if m == 1 && cat.carousel_idx == 0 {
                            cat.carousel_idx = 1;
                        }
                    }
                    mx += mw + 8.0 * u;
                }

                let cy = y + 38.0 * u;
                let ch = 176.0 * u;
                let gap = 8.0 * u;
                #[allow(clippy::type_complexity)]
                let items: [(&str, &str, &str, MaterialShape, [f32; 4], [f32; 4]); 5] = [
                    (
                        "Alpine Dawn",
                        "Dolomites • 4K HDR",
                        "landscape",
                        MaterialShape::Sunny,
                        argb_hex(0x3B236A),
                        argb_hex(0xD0BCFF),
                    ),
                    (
                        "Pacific Swell",
                        "Big Sur Coastline",
                        "waves",
                        MaterialShape::Clover4Leaf,
                        argb_hex(0x1E3F5A),
                        argb_hex(0x9CCAFF),
                    ),
                    (
                        "Redwood Mist",
                        "Humboldt Reserve",
                        "park",
                        MaterialShape::Flower,
                        argb_hex(0x214738),
                        argb_hex(0xA6F2D1),
                    ),
                    (
                        "Sahara Gold",
                        "Erg Chebbi Dunes",
                        "sunny",
                        MaterialShape::SoftBurst,
                        argb_hex(0x5B3224),
                        argb_hex(0xFFB59A),
                    ),
                    (
                        "Kyoto Bloom",
                        "Arashiyama Grove",
                        "local_cafe",
                        MaterialShape::Cookie9Sided,
                        argb_hex(0x592642),
                        argb_hex(0xFFD8E8),
                    ),
                ];
                let n = items.len();
                let focus = self
                    .state
                    .spring_retarget(
                        WidgetId::from_label("carousel_pos"),
                        0,
                        cat.carousel_idx as f32,
                        Tempo::Default,
                        Track::Spatial,
                    )
                    .clamp(0.0, (n - 1) as f32);

                let mut raw_weights = [0.0f32; 5];
                for (i, slot) in raw_weights.iter_mut().enumerate() {
                    let dist = (i as f32 - focus).abs();
                    *slot = match cat.carousel_mode {
                        0 => {
                            if dist < 1.0 {
                                lerp(4.2, 2.2, dist)
                            } else if dist < 2.0 {
                                lerp(2.2, 0.95, dist - 1.0)
                            } else {
                                lerp(0.95, 0.65, (dist - 2.0).min(1.0))
                            }
                        }
                        1 => {
                            if dist < 1.0 {
                                lerp(4.6, 1.05, dist)
                            } else {
                                lerp(1.05, 0.55, (dist - 1.0).min(1.0))
                            }
                        }
                        _ => {
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
                    let r = (28.0 * u).min(cw * 0.5);
                    let card_rect = Rect::new(cx0, cy, cw, ch);
                    let cid = WidgetId::from_label("car_page").with_index(i);

                    self.renderer.rect(cx0, cy, cw, ch, bg_col, r4(r));
                    self.draw_state_layer(cid, card_rect, r4(r), pal.text);
                    self.draw_ripples(cid, card_rect);

                    self.renderer.push_clip([cx0, cy, cw, ch]);
                    self.renderer.m3_shape(
                        cx0 + cw * 0.68,
                        cy + ch * 0.36,
                        (cw * 0.78).min(124.0 * u).max(44.0 * u),
                        shape,
                        0.18 * i as f32 + self.state.t * 0.08,
                        with_alpha(accent_col, 0.24),
                        with_alpha(accent_col, 0.45),
                        1.5 * u,
                    );
                    self.renderer.circle(
                        cx0 + cw * 0.22,
                        cy + ch * 0.24,
                        28.0 * u,
                        with_alpha(accent_col, 0.14),
                    );
                    self.renderer.rect(
                        cx0,
                        cy + ch - 68.0 * u,
                        cw,
                        68.0 * u,
                        with_alpha(argb_hex(0x141218), 0.68),
                        [0.0, 0.0, r, r],
                    );
                    self.renderer.circle(
                        cx0 + (24.0 * u).min(cw * 0.5),
                        cy + 24.0 * u,
                        14.0 * u,
                        with_alpha(argb_hex(0x141218), 0.50),
                    );
                    icon_c(
                        self.renderer,
                        ic,
                        cx0 + (24.0 * u).min(cw * 0.5),
                        cy + 24.0 * u,
                        16.0 * u,
                        accent_col,
                    );

                    let text_alpha = ((cw - 76.0 * u) / (44.0 * u)).clamp(0.0, 1.0);
                    if text_alpha > 0.05 {
                        self.renderer.text(
                            title,
                            cx0 + 14.0 * u,
                            cy + ch - 52.0 * u,
                            ts(M3_TITLE_MEDIUM, with_alpha(argb_hex(0xE6E0E9), text_alpha), u),
                        );
                        self.renderer.text(
                            sub,
                            cx0 + 14.0 * u,
                            cy + ch - 28.0 * u,
                            ts(M3_BODY_SMALL, with_alpha(accent_col, text_alpha), u),
                        );
                    }
                    self.renderer.pop_clip();

                    self.renderer.stroke(
                        cx0,
                        cy,
                        cw,
                        ch,
                        with_alpha(accent_col, if cat.carousel_idx == i { 0.65 } else { 0.22 }),
                        r4(r),
                        1.0 * u,
                    );
                    let resp = self.register_zone(
                        cid,
                        card_rect,
                        pal.text,
                        ZoneKind::SwipeHorizontalDrag,
                        false,
                    );
                    if resp.clicked {
                        cat.carousel_idx = i;
                    } else if resp.dragged {
                        cat.carousel_drag_accum += resp.drag_delta_x;
                        if cat.carousel_drag_accum <= -42.0 * u && cat.carousel_idx + 1 < n {
                            cat.carousel_idx += 1;
                            cat.carousel_drag_accum = 0.0;
                        } else if cat.carousel_drag_accum >= 42.0 * u && cat.carousel_idx > 0 {
                            cat.carousel_idx -= 1;
                            cat.carousel_drag_accum = 0.0;
                        }
                    }
                    cx0 += cw + gap;
                }
            }
            ComponentId::DatePicker => {
                self.date_picker(
                    WidgetId::from_label("date_picker_dialog"),
                    x,
                    y,
                    w,
                    &mut cat.date_picker,
                );
            }
            ComponentId::TimePicker => {
                self.time_picker(
                    WidgetId::from_label("time_picker_dialog"),
                    x,
                    y,
                    w,
                    &mut cat.time_picker,
                );
            }
            ComponentId::Tooltip => {
                let mut tip_chip = ChipProps::new(
                    if cat.tooltip_expanded {
                        "Hide Rich Tooltip"
                    } else {
                        "Show Rich Tooltip"
                    },
                    cat.tooltip_expanded,
                );
                let (tw_c, resp, _) = self.chip(
                    WidgetId::from_label("tooltip_toggle"),
                    x + 16.0 * u,
                    y,
                    &mut tip_chip,
                    false,
                );
                if resp.clicked {
                    cat.tooltip_expanded = !cat.tooltip_expanded;
                }
                self.renderer.rect(
                    x + 28.0 * u + tw_c,
                    y + 2.0 * u,
                    148.0 * u,
                    28.0 * u,
                    pal.inverse,
                    r4(6.0 * u),
                );
                text_c(
                    self.renderer,
                    "Plain tooltip hint",
                    x + 102.0 * u + tw_c,
                    y + 8.0 * u,
                    ts(M3_BODY_SMALL, pal.on_inverse, u),
                );

                if cat.tooltip_expanded {
                    let ry = y + 38.0 * u;
                    let rw = (330.0 * u).min(w - 16.0 * u);
                    self.renderer.rect_styled(
                        x + 16.0 * u,
                        ry,
                        rw,
                        86.0 * u,
                        pal.card_hi,
                        pal.outline_variant,
                        1.0 * u,
                        r4(12.0 * u),
                        0.0,
                    );
                    self.renderer.text(
                        "Rich Expressive Tooltip",
                        x + 32.0 * u,
                        ry + 10.0 * u,
                        ts(M3_TITLE_SMALL, pal.text, u),
                    );
                    self.renderer.text(
                        "Provides contextual details and an inline action.",
                        x + 32.0 * u,
                        ry + 30.0 * u,
                        ts(M3_BODY_SMALL, pal.sub, u),
                    );
                    self.button(
                        WidgetId::from_label("tooltip_action"),
                        x + 24.0 * u,
                        ry + 48.0 * u,
                        &ButtonProps::text("Learn more").with_size(Size::Xs),
                    );
                }
            }
            ComponentId::Snackbar => {
                if !cat.snackbar_visible {
                    if self
                        .button(
                            WidgetId::from_label("reopen_snackbar"),
                            x,
                            y + 12.0 * u,
                            &ButtonProps::tonal("Show Snackbar"),
                        )
                        .1
                        .clicked
                    {
                        cat.snackbar_visible = true;
                        cat.snackbar_undone = false;
                    }
                } else {
                    self.renderer
                        .rect(x, y + 8.0 * u, w, 48.0 * u, pal.inverse, r4(8.0 * u));
                    let msg = if cat.snackbar_undone {
                        "Archive undone • Project restored"
                    } else {
                        "Project archived to workspace"
                    };
                    self.renderer.text(
                        msg,
                        x + 16.0 * u,
                        y + 22.0 * u,
                        ts(M3_BODY_MEDIUM, pal.on_inverse, u),
                    );
                    let st = ts(M3_LABEL_LARGE, pal.inverse_primary, u);
                    let tw = self
                        .renderer
                        .text_width("Undo", st.px, st.medium, st.tracking_em);
                    let undo_id = WidgetId::from(96u32);
                    let rect = Rect::new(
                        x + w - 48.0 * u - tw - 16.0 * u,
                        y + 8.0 * u,
                        tw + 16.0 * u,
                        48.0 * u,
                    );
                    self.draw_state_layer(undo_id, rect, r4(8.0 * u), pal.inverse_primary);
                    self.draw_ripples(undo_id, rect);
                    self.renderer
                        .text("Undo", rect.x + 8.0 * u, y + 22.0 * u, st);
                    if self
                        .register_zone(
                            undo_id,
                            rect,
                            pal.inverse_primary,
                            ZoneKind::Tap { bounded: true },
                            false,
                        )
                        .clicked
                    {
                        cat.snackbar_undone = true;
                    }
                    let close_id = WidgetId::from(97u32);
                    let close_rect = Rect::new(x + w - 42.0 * u, y + 14.0 * u, 36.0 * u, 36.0 * u);
                    icon_c(
                        self.renderer,
                        "close",
                        x + w - 24.0 * u,
                        y + 32.0 * u,
                        18.0 * u,
                        pal.on_inverse,
                    );
                    if self
                        .register_zone(
                            close_id,
                            close_rect,
                            pal.on_inverse,
                            ZoneKind::Tap { bounded: false },
                            false,
                        )
                        .clicked
                    {
                        cat.snackbar_visible = false;
                    }
                }
            }
            ComponentId::Badge => {
                let icons = ["notifications", "mail", "folder"];
                for (i, ic) in icons.into_iter().enumerate() {
                    let bx = x + 4.0 * u + i as f32 * 58.0 * u;
                    let by = y + 8.0 * u;
                    if self
                        .icon_button(
                            WidgetId::from_label("badge_btn").with_index(i),
                            bx,
                            by,
                            ic,
                            &IconButtonProps::tonal(),
                        )
                        .clicked
                    {
                        cat.badge_counts[i] = (cat.badge_counts[i] + 1) % 105;
                    }
                    let count = cat.badge_counts[i];
                    if i == 0 && count > 0 {
                        self.renderer
                            .circle(bx + 32.0 * u, by + 6.0 * u, 4.0 * u, pal.error);
                    } else if count > 0 && count < 10 {
                        self.renderer
                            .circle(bx + 34.0 * u, by + 6.0 * u, 9.0 * u, pal.error);
                        text_c(
                            self.renderer,
                            &format!("{count}"),
                            bx + 34.0 * u,
                            by - 2.0 * u,
                            ts(M3_LABEL_SMALL, pal.on_error, u),
                        );
                    } else if count >= 10 {
                        let lbl = if count > 99 {
                            "99+".to_string()
                        } else {
                            format!("{count}")
                        };
                        self.renderer
                            .rect(bx + 24.0 * u, by - 3.0 * u, 28.0 * u, 18.0 * u, pal.error, r4(9.0 * u));
                        text_c(
                            self.renderer,
                            &lbl,
                            bx + 38.0 * u,
                            by - 2.0 * u,
                            ts(M3_LABEL_SMALL, pal.on_error, u),
                        );
                    }
                }
                self.renderer.text(
                    "Tap icons to increment badges",
                    x + 184.0 * u,
                    y + 18.0 * u,
                    ts(M3_BODY_MEDIUM, pal.sub, u),
                );
            }
            ComponentId::ListItem => {
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
                        "Oct 16",
                        88.0,
                    ),
                ];
                let mut ry = y;
                for (i, (sh, ic, head, support, tr, h)) in rows.into_iter().enumerate() {
                    let hh = h * u;
                    let sel = cat.list_selected[i];
                    let radii = if sel {
                        r4(16.0 * u)
                    } else {
                        match i {
                            0 => [16.0 * u, 16.0 * u, 4.0 * u, 4.0 * u],
                            1 => r4(4.0 * u),
                            _ => [4.0 * u, 4.0 * u, 16.0 * u, 16.0 * u],
                        }
                    };
                    let rect = Rect::new(x, ry, w, hh);
                    let fill = if sel {
                        pal.secondary_container
                    } else {
                        pal.card_hi
                    };
                    self.renderer.rect(x, ry, w, hh, fill, radii);
                    let a = WidgetId::from(110 + i as u32);
                    self.draw_state_layer(a, rect, radii, pal.text);
                    self.draw_ripples(a, rect);
                    self.renderer.m3_shape(
                        x + 28.0 * u,
                        ry + hh * 0.5,
                        36.0 * u,
                        sh,
                        0.0,
                        pal.primary_container,
                        [0.0; 4],
                        0.0,
                    );
                    icon_c(
                        self.renderer,
                        if sel { "check" } else { ic },
                        x + 28.0 * u,
                        ry + hh * 0.5,
                        18.0 * u,
                        pal.on_primary_container,
                    );
                    self.renderer.text(
                        head,
                        x + 56.0 * u,
                        ry + if h >= 88.0 { 12.0 } else { 16.0 } * u,
                        ts(M3_BODY_LARGE, pal.text, u),
                    );
                    if !support.is_empty() {
                        self.renderer.text(
                            support,
                            x + 56.0 * u,
                            ry + if h >= 88.0 { 36.0 } else { 40.0 } * u,
                            ts(M3_BODY_MEDIUM, pal.sub, u),
                        );
                    }
                    if h >= 88.0 {
                        self.renderer.text(
                            "Second line of supporting description copy.",
                            x + 56.0 * u,
                            ry + 56.0 * u,
                            ts(M3_BODY_MEDIUM, pal.sub, u),
                        );
                    }
                    let style = ts(M3_LABEL_SMALL, pal.sub, u);
                    let tw = self
                        .renderer
                        .text_width(tr, style.px, style.medium, style.tracking_em);
                    self.renderer
                        .text(tr, x + w - 16.0 * u - tw, ry + 18.0 * u, style);
                    if self
                        .register_zone(a, rect, pal.text, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.list_selected[i] = !cat.list_selected[i];
                    }
                    ry += hh + 2.0 * u;
                }
            }
            ComponentId::Tabs => {
                let items = [
                    ("code", "Code"),
                    ("terminal", "Build"),
                    ("send", "Deploy"),
                    ("description", "Docs"),
                ];
                let tw2 = w / 4.0;
                for (i, (ic, lb)) in items.into_iter().enumerate() {
                    let tx = x + i as f32 * tw2;
                    let sel = cat.primary_tab == i;
                    let col = if sel { pal.primary } else { pal.sub };
                    let rect = Rect::new(tx, y, tw2, 60.0 * u);
                    let tid = WidgetId::from_label("ptab").with_index(i);
                    self.draw_state_layer(tid, rect, r4(8.0 * u), pal.text);
                    self.draw_ripples(tid, rect);
                    icon_c(self.renderer, ic, tx + tw2 * 0.5, y + 18.0 * u, 20.0 * u, col);
                    text_c(
                        self.renderer,
                        lb,
                        tx + tw2 * 0.5,
                        y + 34.0 * u,
                        ts(M3_TITLE_SMALL, col, u),
                    );
                    if self
                        .register_zone(tid, rect, pal.text, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.primary_tab = i;
                    }
                }
                let tab_pos = self.state.spring_retarget(
                    WidgetId::from_label("ptab_ind"),
                    0,
                    cat.primary_tab as f32,
                    Tempo::Default,
                    Track::Spatial,
                );
                let ind_cx = x + (tab_pos + 0.5) * tw2;
                self.renderer.rect(
                    ind_cx - 24.0 * u,
                    y + 57.0 * u,
                    48.0 * u,
                    3.0 * u,
                    pal.primary,
                    [3.0 * u, 3.0 * u, 0.0, 0.0],
                );
                self.renderer
                    .rect(x, y + 60.0 * u, w, 1.0 * u, pal.outline_variant, r4(0.0));

                let sec_items = ["Overview", "Specifications", "Guidelines"];
                let sw = w / 3.0;
                let sy = y + 70.0 * u;
                for (i, lb) in sec_items.into_iter().enumerate() {
                    let tx = x + i as f32 * sw;
                    let sel = cat.secondary_tab == i;
                    let col = if sel { pal.text } else { pal.sub };
                    let rect = Rect::new(tx, sy, sw, 44.0 * u);
                    let sid = WidgetId::from_label("stab").with_index(i);
                    self.draw_state_layer(sid, rect, r4(8.0 * u), pal.text);
                    self.draw_ripples(sid, rect);
                    text_c(
                        self.renderer,
                        lb,
                        tx + sw * 0.5,
                        sy + 12.0 * u,
                        ts(M3_TITLE_SMALL, col, u),
                    );
                    if self
                        .register_zone(sid, rect, pal.text, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.secondary_tab = i;
                    }
                }
                let sec_pos = self.state.spring_retarget(
                    WidgetId::from_label("stab_ind"),
                    0,
                    cat.secondary_tab as f32,
                    Tempo::Default,
                    Track::Spatial,
                );
                self.renderer.rect(
                    x + sec_pos * sw,
                    sy + 42.0 * u,
                    sw,
                    2.0 * u,
                    pal.primary,
                    r4(1.0 * u),
                );
                self.renderer
                    .rect(x, sy + 44.0 * u, w, 1.0 * u, pal.outline_variant, r4(0.0));
            }
            ComponentId::SegmentedButton => {
                let labels = ["Expressive", "Standard", "Custom"];
                let n = labels.len();
                let h = 40.0 * u;
                let mut weights = [1.0f32; 3];
                for (i, wt) in weights.iter_mut().enumerate() {
                    let sid = WidgetId::from_label("seg_btn").with_index(i);
                    let sel_t = self
                        .state
                        .spring_retarget(
                            sid,
                            0,
                            f32::from(cat.segmented_sel == i),
                            Tempo::Fast,
                            Track::Spatial,
                        )
                        .clamp(0.0, 1.2);
                    let press_t = self
                        .state
                        .spring_retarget(
                            sid,
                            1,
                            f32::from(self.is_pressed(sid)),
                            Tempo::Fast,
                            Track::Spatial,
                        )
                        .clamp(0.0, 1.2);
                    *wt += 0.08 * sel_t + 0.10 * press_t;
                }
                let weight_sum: f32 = weights.iter().sum();
                let mut sx = x;
                for (i, lb) in labels.into_iter().enumerate() {
                    let sid = WidgetId::from_label("seg_btn").with_index(i);
                    let sw = w * (weights[i] / weight_sum);
                    let sel_t = self.state.spring(sid, 0, 0.0, true).clamp(0.0, 1.0);
                    let end = 20.0 * u;
                    let seg_r = if i == 0 {
                        [end, 0.0, 0.0, end]
                    } else if i == n - 1 {
                        [0.0, end, end, 0.0]
                    } else {
                        r4(0.0)
                    };
                    let fill = with_alpha(pal.secondary_container, sel_t);
                    self.renderer.rect_styled(
                        sx,
                        y + 4.0 * u,
                        sw,
                        h,
                        fill,
                        pal.outline,
                        1.0 * u,
                        seg_r,
                        0.0,
                    );
                    let rect = Rect::new(sx, y + 4.0 * u, sw, h);
                    self.draw_state_layer(sid, rect, seg_r, pal.text);
                    self.draw_ripples(sid, rect);
                    let ink = mix(pal.text, pal.on_secondary_container, sel_t);
                    let style = ts(M3_LABEL_LARGE, ink, u);
                    let ltw = self
                        .renderer
                        .text_width(lb, style.px, style.medium, style.tracking_em);
                    let icon_w = 22.0 * u * sel_t;
                    let start = sx + (sw - ltw - icon_w) * 0.5;
                    if sel_t > 0.08 {
                        icon_c(
                            self.renderer,
                            "check",
                            start + 8.0 * u,
                            y + 24.0 * u,
                            18.0 * u * sel_t,
                            with_alpha(ink, sel_t),
                        );
                    }
                    self.renderer
                        .text(lb, start + icon_w, y + 24.0 * u - style.line_h * 0.5, style);
                    if self
                        .register_zone(sid, rect, pal.text, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        cat.segmented_sel = i;
                    }
                    sx += sw;
                }
            }
            ComponentId::Divider => {
                self.renderer.text(
                    "Full-bleed & Inset Dividers",
                    x,
                    y,
                    ts(M3_LABEL_SMALL, pal.sub, u),
                );
                self.renderer.rect(
                    x,
                    y + 22.0 * u,
                    w,
                    1.0 * u,
                    pal.outline_variant,
                    r4(0.5 * u),
                );
                self.renderer.rect(
                    x + 56.0 * u,
                    y + 40.0 * u,
                    (w - 72.0 * u).max(40.0 * u),
                    1.0 * u,
                    pal.outline_variant,
                    r4(0.5 * u),
                );
            }
            ComponentId::PullToRefresh => {
                let rect = Rect::new(x, y, w, 68.0 * u);
                self.pull_to_refresh(
                    WidgetId::from_label("pull_to_refresh"),
                    rect,
                    &mut cat.pull_refresh,
                );
            }
            ComponentId::SwipeToDismiss => {
                self.renderer.text(
                    "Drag cards left (Delete) or right (Archive):",
                    x,
                    y,
                    ts(M3_LABEL_MEDIUM, pal.sub, u),
                );
                let r0 = Rect::new(x, y + 22.0 * u, w, 64.0 * u);
                self.swipe_to_dismiss(
                    WidgetId::from_label("swipe_0"),
                    r0,
                    &mut cat.swipe_items[0],
                );
                let r1 = Rect::new(x, y + 94.0 * u, w, 64.0 * u);
                self.swipe_to_dismiss(
                    WidgetId::from_label("swipe_1"),
                    r1,
                    &mut cat.swipe_items[1],
                );
            }
            ComponentId::Toolbar => {
                let icons = ["edit", "palette", "brush", "share", "more_vert"];
                let tw3 = icons.len() as f32 * 44.0 * u + 16.0 * u;
                let th = 56.0 * u;
                self.renderer.rect(
                    x,
                    y + 4.0 * u,
                    tw3,
                    th,
                    with_alpha(pal.bg, 0.7),
                    r4(28.0 * u),
                );
                self.renderer
                    .rect(x, y + 2.0 * u, tw3, th, pal.primary_container, r4(28.0 * u));
                let tool_pos = self.state.spring_retarget(
                    WidgetId::from_label("toolbar_pos"),
                    0,
                    cat.toolbar_sel as f32,
                    Tempo::Fast,
                    Track::Spatial,
                );
                self.renderer.rect(
                    x + 8.0 * u + tool_pos * 44.0 * u,
                    y + 10.0 * u,
                    40.0 * u,
                    40.0 * u,
                    pal.primary,
                    r4(20.0 * u),
                );
                for (i, ic) in icons.into_iter().enumerate() {
                    let bx = x + 8.0 * u + i as f32 * 44.0 * u;
                    let by = y + 10.0 * u;
                    let id = WidgetId::from(40 + i as u32);
                    let rect = Rect::new(bx, by, 40.0 * u, 40.0 * u);
                    self.draw_state_layer(id, rect, r4(20.0 * u), pal.on_primary_container);
                    self.draw_ripples(id, rect);
                    icon_c(
                        self.renderer,
                        ic,
                        bx + 20.0 * u,
                        by + 20.0 * u,
                        22.0 * u,
                        if cat.toolbar_sel == i {
                            pal.on_primary
                        } else {
                            pal.on_primary_container
                        },
                    );
                    if self
                        .register_zone(
                            id,
                            rect,
                            pal.on_primary_container,
                            ZoneKind::Tap { bounded: true },
                            false,
                        )
                        .clicked
                    {
                        cat.toolbar_sel = i;
                    }
                }
                if self
                    .fab_styled(
                        49u32,
                        x + tw3 + 12.0 * u,
                        y + 2.0 * u,
                        56.0 * u,
                        16.0 * u,
                        pal.secondary_container,
                        pal.on_secondary_container,
                        "add",
                        24.0 * u,
                    )
                    .1
                    .clicked
                {
                    cat.toolbar_sel = (cat.toolbar_sel + 1) % icons.len();
                }
            }
        }
    }
}
