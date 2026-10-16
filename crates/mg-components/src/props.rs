use mg_tokens::shape::MaterialShape;
use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_PI_2, TAU};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Size {
    Xs,
    S,
    M,
    L,
    Xl,
}

impl Size {
    /// Official Material 3 Expressive button height in dp for each size tier.
    pub const fn button_height_dp(self) -> f32 {
        match self {
            Self::Xs => 32.0,
            Self::S => 40.0,
            Self::M => 56.0,
            Self::L => 96.0,
            Self::Xl => 136.0,
        }
    }

    /// Official Material 3 Expressive horizontal padding in dp for each size tier.
    pub const fn horizontal_padding_dp(self) -> f32 {
        match self {
            Self::Xs => 12.0,
            Self::S => 16.0,
            Self::M => 24.0,
            Self::L => 48.0,
            Self::Xl => 64.0,
        }
    }

    /// Official Material 3 Expressive icon size in dp for each size tier.
    pub const fn icon_dp(self) -> f32 {
        match self {
            Self::Xs | Self::S => 18.0,
            Self::M => 24.0,
            Self::L => 32.0,
            Self::Xl => 40.0,
        }
    }

    /// Official Material 3 Expressive pressed corner radius in dp for each size tier.
    pub const fn pressed_radius_dp(self) -> f32 {
        match self {
            Self::Xs | Self::S => 8.0,
            Self::M => 12.0,
            Self::L | Self::Xl => 16.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Variant {
    Elevated,
    Filled,
    Tonal,
    Outlined,
    Text,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ButtonProps {
    pub variant: Variant,
    pub size: Size,
    pub enabled: bool,
    pub label: String,
    pub icon: Option<String>,
    pub morph_on_press: bool,
}

impl ButtonProps {
    pub fn new(variant: Variant, label: impl Into<String>) -> Self {
        Self {
            variant,
            size: Size::S,
            enabled: true,
            label: label.into(),
            icon: None,
            morph_on_press: true,
        }
    }

    pub fn filled(label: impl Into<String>) -> Self {
        Self::new(Variant::Filled, label)
    }

    pub fn tonal(label: impl Into<String>) -> Self {
        Self::new(Variant::Tonal, label)
    }

    pub fn outlined(label: impl Into<String>) -> Self {
        Self::new(Variant::Outlined, label)
    }

    pub fn elevated(label: impl Into<String>) -> Self {
        Self::new(Variant::Elevated, label)
    }

    pub fn text(label: impl Into<String>) -> Self {
        Self::new(Variant::Text, label)
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FabSize {
    Small,
    Medium,
    Large,
}

impl FabSize {
    pub const fn container_dp(self) -> f32 {
        match self {
            Self::Small => 40.0,
            Self::Medium => 56.0,
            Self::Large => 80.0,
        }
    }

    pub const fn corner_radius_dp(self) -> f32 {
        match self {
            Self::Small => 12.0,
            Self::Medium => 16.0,
            Self::Large => 24.0,
        }
    }

    pub const fn icon_dp(self) -> f32 {
        match self {
            Self::Small | Self::Medium => 24.0,
            Self::Large => 36.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FabProps {
    pub size: FabSize,
    pub expanded: bool,
    pub menu_open: bool,
    pub menu_items: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SplitButtonProps {
    pub main_label: String,
    pub leading_icon: Option<String>,
    pub tonal: bool,
    pub menu_open: bool,
    pub enabled: bool,
    pub menu_items: Vec<String>,
    pub selected_item: usize,
}

impl SplitButtonProps {
    pub fn new(main_label: impl Into<String>, menu_items: Vec<String>) -> Self {
        Self {
            main_label: main_label.into(),
            leading_icon: None,
            tonal: false,
            menu_open: false,
            enabled: true,
            menu_items,
            selected_item: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ButtonGroupProps {
    pub connected: bool,
    pub multi_select: bool,
    pub selected: Vec<usize>,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconButtonVariant {
    Standard,
    Filled,
    Tonal,
    Outlined,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IconButtonProps {
    pub variant: IconButtonVariant,
    pub toggleable: bool,
    pub checked: bool,
    pub enabled: bool,
    pub wide: bool,
}

impl IconButtonProps {
    pub const fn standard() -> Self {
        Self {
            variant: IconButtonVariant::Standard,
            toggleable: false,
            checked: false,
            enabled: true,
            wide: false,
        }
    }

    pub const fn filled() -> Self {
        Self {
            variant: IconButtonVariant::Filled,
            ..Self::standard()
        }
    }

    pub const fn tonal() -> Self {
        Self {
            variant: IconButtonVariant::Tonal,
            ..Self::standard()
        }
    }

    pub const fn outlined() -> Self {
        Self {
            variant: IconButtonVariant::Outlined,
            ..Self::standard()
        }
    }

    pub const fn with_wide(mut self, wide: bool) -> Self {
        self.wide = wide;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckState {
    Off,
    On,
    Indeterminate,
}

impl CheckState {
    pub const fn cycle(self) -> Self {
        match self {
            Self::Off => Self::On,
            Self::On => Self::Indeterminate,
            Self::Indeterminate => Self::Off,
        }
    }

    pub const fn toggle_binary(self) -> Self {
        match self {
            Self::On | Self::Indeterminate => Self::Off,
            Self::Off => Self::On,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckboxProps {
    pub state: CheckState,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SwitchProps {
    pub on: bool,
    pub enabled: bool,
    pub with_icon: bool,
}

impl SwitchProps {
    pub const fn new(on: bool) -> Self {
        Self {
            on,
            enabled: true,
            with_icon: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RadioProps {
    pub selected: bool,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SliderProps {
    pub value: f32,
    pub range: (f32, f32),
    pub steps: u32,
    pub enabled: bool,
    pub vertical: bool,
    pub range_end: Option<f32>,
}

impl SliderProps {
    pub const fn new(value: f32) -> Self {
        Self {
            value,
            range: (0.0, 1.0),
            steps: 0,
            enabled: true,
            vertical: false,
            range_end: None,
        }
    }

    pub const fn with_range(value: f32, min: f32, max: f32) -> Self {
        Self {
            value,
            range: (min, max),
            steps: 0,
            enabled: true,
            vertical: false,
            range_end: None,
        }
    }

    pub fn normalized(&self) -> f32 {
        let span = (self.range.1 - self.range.0).max(f32::EPSILON);
        ((self.value - self.range.0) / span).clamp(0.0, 1.0)
    }

    pub fn set_from_normalized(&mut self, t: f32) {
        let mut t = t.clamp(0.0, 1.0);
        if self.steps > 0 {
            let intervals = (self.steps + 1) as f32;
            t = (t * intervals).round() / intervals;
        }
        self.value = self.range.0 + (self.range.1 - self.range.0) * t;
    }
}

impl Default for SliderProps {
    fn default() -> Self {
        Self::new(0.5)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgressProps {
    pub progress: Option<f32>,
    pub wavy: bool,
}

impl ProgressProps {
    pub const STROKE_WIDTH_DP: f32 = 4.0;
    pub const TRACK_GAP_DP: f32 = 4.0;
    pub const STOP_SIZE_DP: f32 = 4.0;
    pub const LINEAR_WAVELENGTH_DP: f32 = 40.0;
    pub const LINEAR_INDETERMINATE_WAVELENGTH_DP: f32 = 20.0;
    pub const CIRCULAR_SIZE_DP: f32 = 48.0;
    pub const WAVE_AMPLITUDE_DP: f32 = 3.0;
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadingProps {
    pub contained: bool,
}

impl LoadingProps {
    pub const DEFAULT_CONTAINER_DP: f32 = 48.0;
    pub const ACTIVE_INDICATOR_DP: f32 = 38.0;
    pub const MORPH_INTERVAL_SECS: f32 = 0.65;
    pub const FULL_ROTATION_SECS: f32 = 4.666;
    pub const STEP_ROTATION_RAD: f32 = std::f32::consts::FRAC_PI_4;

    pub const fn default_polygons() -> &'static [MaterialShape; 7] {
        &MaterialShape::LOADING_SEQUENCE
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChipProps {
    pub label: String,
    pub selected: bool,
    pub enabled: bool,
    pub with_icon: bool,
    pub leading_icon: Option<String>,
    pub trailing_icon: Option<String>,
}

impl ChipProps {
    pub fn new(label: impl Into<String>, selected: bool) -> Self {
        Self {
            label: label.into(),
            selected,
            enabled: true,
            with_icon: false,
            leading_icon: None,
            trailing_icon: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardVariant {
    Elevated,
    Filled,
    Outlined,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CardProps {
    pub variant: CardVariant,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialogKind {
    Alert,
    Basic,
    FullScreen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogState {
    pub open: bool,
    pub kind: DialogKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SheetKind {
    ModalBottom,
    Side,
    Scaffold,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SheetState {
    pub open: bool,
    pub kind: SheetKind,
    pub fraction: f32,
    pub skip_partial: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavProps {
    pub items: Vec<String>,
    pub selected: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawerKind {
    Modal,
    Dismissible,
    Permanent,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DrawerState {
    pub open: bool,
    pub kind: DrawerKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppBarKind {
    Small,
    CenterAligned,
    Medium,
    Large,
    Flexible,
    ExitUntilCollapsed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScaffoldProps {
    pub app_bar: AppBarKind,
    pub fab_present: bool,
    pub snackbar_present: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchProps {
    pub query: String,
    pub expanded: bool,
    pub docked: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextFieldProps {
    pub text: String,
    pub label: String,
    pub placeholder: String,
    pub supporting: Option<String>,
    pub prefix: Option<String>,
    pub leading_icon: Option<String>,
    pub trailing_icon: Option<String>,
    pub outlined: bool,
    pub tonal: bool,
    pub secure: bool,
    pub password_visible: bool,
    pub clearable: bool,
    pub numeric_only: bool,
    pub max_length: Option<usize>,
    pub error: Option<String>,
    pub enabled: bool,
    pub cursor: usize,
}

impl TextFieldProps {
    pub fn filled(label: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor = text.chars().count();
        Self {
            text,
            label: label.into(),
            placeholder: String::new(),
            supporting: None,
            prefix: None,
            leading_icon: None,
            trailing_icon: None,
            outlined: false,
            tonal: false,
            secure: false,
            password_visible: false,
            clearable: true,
            numeric_only: false,
            max_length: None,
            error: None,
            enabled: true,
            cursor,
        }
    }

    pub fn outlined(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            outlined: true,
            ..Self::filled(label, text)
        }
    }

    pub fn password(label: impl Into<String>, text: impl Into<String>, outlined: bool) -> Self {
        Self {
            outlined,
            secure: true,
            password_visible: false,
            clearable: false,
            leading_icon: Some("lock".to_string()),
            ..Self::filled(label, text)
        }
    }

    pub fn char_len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn clamp_cursor(&mut self) {
        let len = self.char_len();
        if self.cursor > len {
            self.cursor = len;
        }
    }

    pub fn byte_index_of_char(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    fn update_length_validation(&mut self) {
        if let Some(max) = self.max_length {
            if self.char_len() > max {
                self.error = Some(format!("Maximum {max} characters"));
            } else {
                self.error = None;
            }
        }
    }

    pub fn insert_char(&mut self, ch: char) -> bool {
        if !self.enabled || ch.is_control() {
            return false;
        }
        if self.numeric_only && !ch.is_ascii_digit() && ch != '/' && ch != '-' && ch != '.' {
            return false;
        }
        if let Some(max) = self.max_length {
            if self.char_len() >= max + 6 {
                return false;
            }
        }
        self.clamp_cursor();
        let byte_idx = self.byte_index_of_char(self.cursor);
        self.text.insert(byte_idx, ch);
        self.cursor += 1;
        self.update_length_validation();
        true
    }

    pub fn insert_str(&mut self, s: &str) -> bool {
        let mut any = false;
        for ch in s.chars() {
            if self.insert_char(ch) {
                any = true;
            }
        }
        any
    }

    pub fn backspace(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        self.clamp_cursor();
        if self.cursor == 0 {
            return false;
        }
        let prev = self.cursor - 1;
        let start = self.byte_index_of_char(prev);
        let end = self.byte_index_of_char(self.cursor);
        self.text.replace_range(start..end, "");
        self.cursor = prev;
        self.update_length_validation();
        true
    }

    pub fn delete_forward(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        self.clamp_cursor();
        let len = self.char_len();
        if self.cursor >= len {
            return false;
        }
        let start = self.byte_index_of_char(self.cursor);
        let end = self.byte_index_of_char(self.cursor + 1);
        self.text.replace_range(start..end, "");
        self.update_length_validation();
        true
    }

    pub fn clear(&mut self) {
        if !self.enabled {
            return;
        }
        self.text.clear();
        self.cursor = 0;
        self.error = None;
    }

    pub fn move_left(&mut self) {
        self.clamp_cursor();
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        self.clamp_cursor();
        let len = self.char_len();
        if self.cursor < len {
            self.cursor += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.char_len();
    }

    pub fn display_text(&self) -> String {
        if self.secure && !self.password_visible {
            "•".repeat(self.char_len())
        } else {
            self.text.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuProps {
    pub open: bool,
    pub items: Vec<String>,
    pub groups: Vec<(String, usize)>,
    pub selected: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CarouselKind {
    MultiBrowse,
    Uncontained,
    Hero,
    CenteredHero,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CarouselState {
    pub kind: CarouselKind,
    pub index: usize,
    pub count: usize,
}

impl CarouselState {
    pub const CORNER_RADIUS_DP: f32 = 28.0;
    pub const ITEM_SPACING_DP: f32 = 8.0;
    pub const MIN_SMALL_ITEM_DP: f32 = 40.0;
    pub const MAX_SMALL_ITEM_DP: f32 = 56.0;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateState {
    pub millis: Option<u64>,
    pub range_end_millis: Option<u64>,
    pub dialog_open: bool,
}

/// Official Material 3 DatePicker entry mode (toggled by the top-right pencil `"edit"` / `"calendar_today"` button).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatePickerEntryMode {
    Calendar,
    Input,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatePickerCalendarMode {
    Day,
    Year,
}

/// Complete stateful Material 3 DatePicker controller supporting Calendar grid, Year picker,
/// and numeric DateInput mode (triggered by the pencil icon `"edit"` <-> `"calendar_today"`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatePickerState {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub view_year: i32,
    pub view_month: u32,
    pub entry_mode: DatePickerEntryMode,
    pub calendar_mode: DatePickerCalendarMode,
    pub input_field: TextFieldProps,
    pub confirmed: (i32, u32, u32),
    pub status_text: Option<String>,
}

impl Default for DatePickerState {
    fn default() -> Self {
        Self::new(2026, 10, 16)
    }
}

impl DatePickerState {
    pub const MONTH_NAMES: [&'static str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    pub const MONTH_SHORT: [&'static str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    pub const WEEKDAY_SHORT: [&'static str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

    pub fn new(year: i32, month: u32, day: u32) -> Self {
        let formatted = Self::format_mm_dd_yyyy(year, month, day);
        let mut input_field = TextFieldProps::outlined("Date", formatted);
        input_field.placeholder = "MM/DD/YYYY".to_string();
        input_field.supporting = Some("Format: MM/DD/YYYY or YYYY-MM-DD".to_string());
        input_field.leading_icon = Some("calendar_today".to_string());
        input_field.numeric_only = true;
        input_field.clearable = true;
        Self {
            year,
            month,
            day,
            view_year: year,
            view_month: month,
            entry_mode: DatePickerEntryMode::Calendar,
            calendar_mode: DatePickerCalendarMode::Day,
            input_field,
            confirmed: (year, month, day),
            status_text: None,
        }
    }

    pub const fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    pub const fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Sakamoto's algorithm: returns `0..=6` where `0 = Sunday`, `6 = Saturday`.
    pub const fn weekday_of(year: i32, month: u32, day: u32) -> usize {
        let t = [0i32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = year;
        if month < 3 {
            y -= 1;
        }
        let m_idx = (month.saturating_sub(1) as usize) % 12;
        let w = (y + y / 4 - y / 100 + y / 400 + t[m_idx] + day as i32) % 7;
        w.rem_euclid(7) as usize
    }

    pub fn format_mm_dd_yyyy(year: i32, month: u32, day: u32) -> String {
        format!("{month:02}/{day:02}/{year:04}")
    }

    pub fn format_headline(&self) -> String {
        let wd = Self::WEEKDAY_SHORT[Self::weekday_of(self.year, self.month, self.day)];
        let m = Self::MONTH_SHORT[(self.month.saturating_sub(1) as usize) % 12];
        format!("{wd}, {m} {}", self.day)
    }

    pub fn format_sub_header(&self) -> String {
        let m = Self::MONTH_NAMES[(self.view_month.saturating_sub(1) as usize) % 12];
        format!("{m} {}", self.view_year)
    }

    pub fn sync_input_from_date(&mut self) {
        self.input_field.text = Self::format_mm_dd_yyyy(self.year, self.month, self.day);
        self.input_field.move_end();
        self.input_field.error = None;
    }

    /// Parses user-entered date string in `MM/DD/YYYY`, `YYYY-MM-DD`, `DD.MM.YYYY`, or 8-digit `MMDDYYYY`.
    pub fn parse_date_str(raw: &str) -> Result<(i32, u32, u32), String> {
        let s = raw.trim();
        if s.is_empty() {
            return Err("Enter date in MM/DD/YYYY".to_string());
        }
        let (y, m, d) = if s.contains('/') {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() != 3 {
                return Err("Use MM/DD/YYYY (e.g. 10/16/2026)".to_string());
            }
            let m: u32 = parts[0].trim().parse().map_err(|_| "Invalid month")?;
            let d: u32 = parts[1].trim().parse().map_err(|_| "Invalid day")?;
            let y: i32 = parts[2].trim().parse().map_err(|_| "Invalid year")?;
            (y, m, d)
        } else if s.contains('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 3 {
                return Err("Use YYYY-MM-DD (e.g. 2026-10-16)".to_string());
            }
            let p0: i32 = parts[0].trim().parse().map_err(|_| "Invalid number")?;
            if p0 > 31 {
                let m: u32 = parts[1].trim().parse().map_err(|_| "Invalid month")?;
                let d: u32 = parts[2].trim().parse().map_err(|_| "Invalid day")?;
                (p0, m, d)
            } else {
                let m = p0 as u32;
                let d: u32 = parts[1].trim().parse().map_err(|_| "Invalid day")?;
                let y: i32 = parts[2].trim().parse().map_err(|_| "Invalid year")?;
                (y, m, d)
            }
        } else if s.contains('.') {
            let parts: Vec<&str> = s.split('.').collect();
            if parts.len() != 3 {
                return Err("Use DD.MM.YYYY (e.g. 16.10.2026)".to_string());
            }
            let d: u32 = parts[0].trim().parse().map_err(|_| "Invalid day")?;
            let m: u32 = parts[1].trim().parse().map_err(|_| "Invalid month")?;
            let y: i32 = parts[2].trim().parse().map_err(|_| "Invalid year")?;
            (y, m, d)
        } else if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
            let m: u32 = s[0..2].parse().map_err(|_| "Invalid month")?;
            let d: u32 = s[2..4].parse().map_err(|_| "Invalid day")?;
            let y: i32 = s[4..8].parse().map_err(|_| "Invalid year")?;
            (y, m, d)
        } else {
            return Err("Format: MM/DD/YYYY (e.g. 10/16/2026)".to_string());
        };

        if !(1900..=2100).contains(&y) {
            return Err("Year must be 1900..2100".to_string());
        }
        if !(1..=12).contains(&m) {
            return Err("Month must be 01..12".to_string());
        }
        let max_d = Self::days_in_month(y, m);
        if !(1..=max_d).contains(&d) {
            return Err(format!(
                "{} {y} has {max_d} days",
                Self::MONTH_NAMES[(m - 1) as usize]
            ));
        }
        Ok((y, m, d))
    }

    pub fn sync_date_from_input(&mut self) -> bool {
        match Self::parse_date_str(&self.input_field.text) {
            Ok((y, m, d)) => {
                self.year = y;
                self.month = m;
                self.day = d;
                self.view_year = y;
                self.view_month = m;
                self.input_field.error = None;
                true
            }
            Err(err) => {
                self.input_field.error = Some(err);
                false
            }
        }
    }

    pub fn toggle_entry_mode(&mut self) {
        match self.entry_mode {
            DatePickerEntryMode::Calendar => {
                self.sync_input_from_date();
                self.calendar_mode = DatePickerCalendarMode::Day;
                self.entry_mode = DatePickerEntryMode::Input;
            }
            DatePickerEntryMode::Input => {
                let _ = self.sync_date_from_input();
                self.entry_mode = DatePickerEntryMode::Calendar;
            }
        }
    }

    pub fn toggle_calendar_mode(&mut self) {
        self.calendar_mode = match self.calendar_mode {
            DatePickerCalendarMode::Day => DatePickerCalendarMode::Year,
            DatePickerCalendarMode::Year => DatePickerCalendarMode::Day,
        };
    }

    pub fn prev_month(&mut self) {
        if self.view_month <= 1 {
            self.view_month = 12;
            self.view_year = (self.view_year - 1).max(1900);
        } else {
            self.view_month -= 1;
        }
    }

    pub fn next_month(&mut self) {
        if self.view_month >= 12 {
            self.view_month = 1;
            self.view_year = (self.view_year + 1).min(2100);
        } else {
            self.view_month += 1;
        }
    }

    pub fn select_day(&mut self, day: u32) {
        let max_d = Self::days_in_month(self.view_year, self.view_month);
        self.year = self.view_year;
        self.month = self.view_month;
        self.day = day.clamp(1, max_d);
        self.sync_input_from_date();
    }

    pub fn select_year(&mut self, year: i32) {
        self.view_year = year.clamp(1900, 2100);
        self.year = self.view_year;
        let max_d = Self::days_in_month(self.year, self.month);
        if self.day > max_d {
            self.day = max_d;
        }
        self.calendar_mode = DatePickerCalendarMode::Day;
        self.sync_input_from_date();
    }

    pub fn confirm(&mut self) -> bool {
        if self.entry_mode == DatePickerEntryMode::Input && !self.sync_date_from_input() {
            return false;
        }
        self.confirmed = (self.year, self.month, self.day);
        self.status_text = Some(format!(
            "Confirmed: {} ({})",
            self.format_headline(),
            Self::format_mm_dd_yyyy(self.year, self.month, self.day)
        ));
        self.entry_mode = DatePickerEntryMode::Calendar;
        self.calendar_mode = DatePickerCalendarMode::Day;
        true
    }

    pub fn cancel(&mut self) {
        let (y, m, d) = self.confirmed;
        self.year = y;
        self.month = m;
        self.day = d;
        self.view_year = y;
        self.view_month = m;
        self.sync_input_from_date();
        self.entry_mode = DatePickerEntryMode::Calendar;
        self.calendar_mode = DatePickerCalendarMode::Day;
        self.status_text = Some("Restored confirmed date".to_string());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeState {
    pub hour: u8,
    pub minute: u8,
    pub input_mode: bool,
    pub dialog_open: bool,
}

/// Complete stateful Material 3 TimePicker controller supporting both analog clock dial
/// and numeric keyboard entry mode (`"keyboard"` <-> `"schedule"`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimePickerState {
    pub hour: u8,
    pub minute: u8,
    pub is_am: bool,
    pub minute_mode: bool,
    pub input_mode: bool,
    pub hour_field: TextFieldProps,
    pub minute_field: TextFieldProps,
    pub confirmed: (u8, u8, bool),
    pub status_text: Option<String>,
}

impl Default for TimePickerState {
    fn default() -> Self {
        Self::new(10, 25, true)
    }
}

impl TimePickerState {
    pub fn new(hour: u8, minute: u8, is_am: bool) -> Self {
        let mut hour_field = TextFieldProps::outlined("Hour", format!("{hour:02}"));
        hour_field.numeric_only = true;
        hour_field.clearable = false;
        hour_field.max_length = Some(2);

        let mut minute_field = TextFieldProps::outlined("Minute", format!("{minute:02}"));
        minute_field.numeric_only = true;
        minute_field.clearable = false;
        minute_field.max_length = Some(2);

        Self {
            hour,
            minute,
            is_am,
            minute_mode: false,
            input_mode: false,
            hour_field,
            minute_field,
            confirmed: (hour, minute, is_am),
            status_text: None,
        }
    }

    pub fn sync_fields_from_time(&mut self) {
        self.hour_field.text = format!("{:02}", self.hour);
        self.hour_field.move_end();
        self.hour_field.error = None;
        self.minute_field.text = format!("{:02}", self.minute);
        self.minute_field.move_end();
        self.minute_field.error = None;
    }

    pub fn sync_time_from_fields(&mut self) -> bool {
        let h_ok = self.hour_field.text.trim().parse::<u8>();
        let m_ok = self.minute_field.text.trim().parse::<u8>();
        match (h_ok, m_ok) {
            (Ok(h), Ok(m)) if (1..=12).contains(&h) && m < 60 => {
                self.hour = h;
                self.minute = m;
                self.hour_field.error = None;
                self.minute_field.error = None;
                true
            }
            _ => {
                self.hour_field.error = Some("01..12".to_string());
                self.minute_field.error = Some("00..59".to_string());
                false
            }
        }
    }

    pub fn toggle_input_mode(&mut self) {
        if self.input_mode {
            let _ = self.sync_time_from_fields();
            self.input_mode = false;
        } else {
            self.sync_fields_from_time();
            self.input_mode = true;
        }
    }

    pub fn dial_angle_rad(&self) -> f32 {
        if self.minute_mode {
            (self.minute as f32 / 60.0) * TAU - FRAC_PI_2
        } else {
            ((self.hour % 12) as f32 / 12.0) * TAU - FRAC_PI_2
        }
    }

    pub fn set_from_dial_point(&mut self, cx: f32, cy: f32, px: f32, py: f32) {
        let dx = px - cx;
        let dy = py - cy;
        if dx.hypot(dy) < 6.0 {
            return;
        }
        let mut ang = dy.atan2(dx) + FRAC_PI_2;
        if ang < 0.0 {
            ang += TAU;
        }
        if self.minute_mode {
            let m = ((ang / TAU) * 60.0).round() as u8 % 60;
            self.minute = m;
        } else {
            let mut h = ((ang / TAU) * 12.0).round() as u8 % 12;
            if h == 0 {
                h = 12;
            }
            self.hour = h;
        }
        self.sync_fields_from_time();
    }

    pub fn format_time(&self) -> String {
        let suffix = if self.is_am { "AM" } else { "PM" };
        format!("{:02}:{:02} {suffix}", self.hour, self.minute)
    }

    pub fn confirm(&mut self) -> bool {
        if self.input_mode && !self.sync_time_from_fields() {
            return false;
        }
        self.confirmed = (self.hour, self.minute, self.is_am);
        self.status_text = Some(format!("Confirmed {}", self.format_time()));
        self.input_mode = false;
        true
    }

    pub fn cancel(&mut self) {
        let (h, m, am) = self.confirmed;
        self.hour = h;
        self.minute = m;
        self.is_am = am;
        self.sync_fields_from_time();
        self.input_mode = false;
        self.status_text = Some("Restored confirmed time".to_string());
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TooltipProps {
    pub text: String,
    pub rich: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnackbarData {
    pub message: String,
    pub action: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BadgeProps {
    pub count: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListItemProps {
    pub headline: String,
    pub supporting: Option<String>,
    pub trailing: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TabsProps {
    pub tabs: Vec<String>,
    pub selected: usize,
    pub secondary: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SegmentedProps {
    pub options: Vec<String>,
    pub selected: Vec<usize>,
    pub multi: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DividerProps {
    pub vertical: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PullRefreshState {
    pub refreshing: bool,
    pub pull_fraction: f32,
    pub refresh_started_at: Option<f32>,
    pub refresh_count: u32,
}

impl Default for PullRefreshState {
    fn default() -> Self {
        Self {
            refreshing: false,
            pull_fraction: 0.0,
            refresh_started_at: None,
            refresh_count: 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DismissState {
    pub dismissed: bool,
    pub direction_start_to_end: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SwipeDismissItem {
    pub id: u32,
    pub headline: String,
    pub supporting: String,
    pub trailing: String,
    pub icon: String,
    pub shape: MaterialShape,
    pub offset_dp: f32,
    pub target_dp: f32,
    pub dismissed: bool,
    pub dismissed_start_to_end: bool,
}

impl SwipeDismissItem {
    pub fn new(
        id: u32,
        headline: impl Into<String>,
        supporting: impl Into<String>,
        trailing: impl Into<String>,
        icon: impl Into<String>,
        shape: MaterialShape,
    ) -> Self {
        Self {
            id,
            headline: headline.into(),
            supporting: supporting.into(),
            trailing: trailing.into(),
            icon: icon.into(),
            shape,
            offset_dp: 0.0,
            target_dp: 0.0,
            dismissed: false,
            dismissed_start_to_end: false,
        }
    }

    pub fn reset(&mut self) {
        self.offset_dp = 0.0;
        self.target_dp = 0.0;
        self.dismissed = false;
        self.dismissed_start_to_end = false;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolbarKind {
    Docked,
    FloatingHorizontal,
    FloatingVertical,
    FlexibleBottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolbarProps {
    pub kind: ToolbarKind,
    pub with_fab: bool,
}
