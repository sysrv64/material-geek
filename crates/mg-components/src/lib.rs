#![allow(
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::collapsible_else_if
)]

#[rustfmt::skip]
pub mod context;
#[rustfmt::skip]
pub mod props;
#[rustfmt::skip]
pub mod showcase;
pub mod terminal_ui;
#[rustfmt::skip]
pub mod widgets;

pub use context::{
    argb_hex, argb_to_f32, eval_indet_segment, icon_c, lerp, loading_morph_state, mix, r4, srgb,
    text_c, ts, with_alpha, ColumnLayout, KeyInput, Palette, Rect, RowLayout, SoftKeyboardMode,
    UiContext, UiState, WidgetId, WidgetResponse, ZoneKind,
};
pub use mg_tokens::elevation::ElevationLevel;
pub use mg_tokens::shape::{MaterialShape, ShapeStyle};
pub use props::*;
use serde::{Deserialize, Serialize};
pub use showcase::{CatalogState, APP_BAR_HEIGHT_DP, NAV_PANE_WIDTH_DP, STATUS_BAR_INSET_DP};
pub use terminal_ui::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentId {
    Button,
    IconButton,
    Fab,
    ExtendedFab,
    FabMenu,
    SplitButton,
    ButtonGroup,
    Card,
    Checkbox,
    Switch,
    RadioButton,
    Slider,
    ProgressIndicator,
    LoadingIndicator,
    AssistChip,
    FilterChip,
    InputChip,
    SuggestionChip,
    Dialog,
    BottomSheet,
    NavigationBar,
    NavigationRail,
    NavigationDrawer,
    Scaffold,
    TopAppBar,
    SearchBar,
    TextField,
    Menu,
    Carousel,
    DatePicker,
    TimePicker,
    Tooltip,
    Snackbar,
    Badge,
    ListItem,
    Tabs,
    SegmentedButton,
    Divider,
    PullToRefresh,
    SwipeToDismiss,
    Toolbar,
}

impl ComponentId {
    pub const ALL: [Self; 41] = [
        Self::Button,
        Self::IconButton,
        Self::Fab,
        Self::ExtendedFab,
        Self::FabMenu,
        Self::SplitButton,
        Self::ButtonGroup,
        Self::Card,
        Self::Checkbox,
        Self::Switch,
        Self::RadioButton,
        Self::Slider,
        Self::ProgressIndicator,
        Self::LoadingIndicator,
        Self::AssistChip,
        Self::FilterChip,
        Self::InputChip,
        Self::SuggestionChip,
        Self::Dialog,
        Self::BottomSheet,
        Self::NavigationBar,
        Self::NavigationRail,
        Self::NavigationDrawer,
        Self::Scaffold,
        Self::TopAppBar,
        Self::SearchBar,
        Self::TextField,
        Self::Menu,
        Self::Carousel,
        Self::DatePicker,
        Self::TimePicker,
        Self::Tooltip,
        Self::Snackbar,
        Self::Badge,
        Self::ListItem,
        Self::Tabs,
        Self::SegmentedButton,
        Self::Divider,
        Self::PullToRefresh,
        Self::SwipeToDismiss,
        Self::Toolbar,
    ];
}

pub fn resting_elevation(id: ComponentId) -> ElevationLevel {
    use ComponentId as C;
    match id {
        C::Dialog | C::DatePicker | C::TimePicker | C::Fab | C::ExtendedFab | C::SearchBar => {
            ElevationLevel::L3
        }
        C::Menu | C::NavigationBar | C::Toolbar | C::Tooltip => ElevationLevel::L2,
        C::Card
        | C::Button
        | C::AssistChip
        | C::FilterChip
        | C::InputChip
        | C::SuggestionChip
        | C::BottomSheet => ElevationLevel::L1,
        _ => ElevationLevel::L0,
    }
}

pub fn default_shape(id: ComponentId) -> ShapeStyle {
    use ComponentId as C;
    match id {
        C::Button => ShapeStyle::Full,
        C::Fab | C::ExtendedFab => ShapeStyle::Large,
        C::Card => ShapeStyle::Medium,
        C::Dialog => ShapeStyle::ExtraLarge,
        C::BottomSheet => ShapeStyle::ExtraLarge,
        C::Snackbar => ShapeStyle::Small,
        _ => ShapeStyle::Medium,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn registry_covers_all_41() {
        assert_eq!(ComponentId::ALL.len(), 41);
        let set: HashSet<usize> = ComponentId::ALL.iter().map(|c| *c as usize).collect();
        assert_eq!(set.len(), 41, "duplicates in registry");
    }

    #[test]
    fn elevations_match_spec_examples() {
        assert_eq!(resting_elevation(ComponentId::Dialog), ElevationLevel::L3);
        assert_eq!(resting_elevation(ComponentId::Menu), ElevationLevel::L2);
        assert_eq!(resting_elevation(ComponentId::Switch), ElevationLevel::L0);
    }

    #[test]
    fn fab_menu_capacity_2_to_6() {
        let p = FabProps {
            size: FabSize::Medium,
            expanded: false,
            menu_open: true,
            menu_items: 4,
        };
        assert!((2..=6).contains(&p.menu_items));
    }

    #[test]
    fn loading_sequence_matches_m3e_spec() {
        let seq = LoadingProps::default_polygons();
        assert_eq!(seq.len(), 7);
        assert_eq!(seq[0], MaterialShape::SoftBurst);
        assert_eq!(seq[6], MaterialShape::Oval);
    }

    #[test]
    fn date_picker_entry_mode_and_numeric_parsing_work() {
        let mut dp = DatePickerState::new(2026, 10, 16);
        assert_eq!(dp.entry_mode, DatePickerEntryMode::Calendar);
        assert_eq!(dp.format_headline(), "Fri, Oct 16");

        // Toggle via pencil button -> Input mode
        dp.toggle_entry_mode();
        assert_eq!(dp.entry_mode, DatePickerEntryMode::Input);
        assert_eq!(dp.input_field.text, "10/16/2026");

        // Enter a new date in digits
        dp.input_field.text = "12/25/2026".to_string();
        assert!(dp.sync_date_from_input());
        assert_eq!((dp.year, dp.month, dp.day), (2026, 12, 25));
        assert_eq!(dp.format_headline(), "Fri, Dec 25");

        // Also supports YYYY-MM-DD and 8-digit MMDDYYYY
        assert_eq!(
            DatePickerState::parse_date_str("2028-02-29"),
            Ok((2028, 2, 29))
        );
        assert!(DatePickerState::parse_date_str("2026-02-29").is_err());
        assert_eq!(
            DatePickerState::parse_date_str("04152027"),
            Ok((2027, 4, 15))
        );
    }

    #[test]
    fn text_field_editing_and_password_masking_work() {
        let mut tf = TextFieldProps::filled("Email", "hello");
        assert_eq!(tf.cursor, 5);
        tf.insert_char('!');
        assert_eq!(tf.text, "hello!");
        tf.backspace();
        assert_eq!(tf.text, "hello");
        tf.move_home();
        tf.insert_str("say ");
        assert_eq!(tf.text, "say hello");

        let mut pw = TextFieldProps::password("Password", "secret", true);
        assert_eq!(pw.display_text(), "••••••");
        pw.password_visible = true;
        assert_eq!(pw.display_text(), "secret");
    }

    #[test]
    fn time_picker_input_and_dial_work() {
        let mut tp = TimePickerState::new(10, 25, true);
        assert!(!tp.input_mode);
        tp.toggle_input_mode();
        assert!(tp.input_mode);
        tp.hour_field.text = "04".to_string();
        tp.minute_field.text = "45".to_string();
        tp.is_am = false;
        assert!(tp.confirm());
        assert_eq!(tp.format_time(), "04:45 PM");
    }

    #[test]
    fn swipe_to_dismiss_and_ui_state_theme_toggle_work() {
        let mut item =
            SwipeDismissItem::new(1, "Card", "Swipe me", "Now", "mail", MaterialShape::Arch);
        item.offset_dp = -120.0;
        item.dismissed = true;
        item.reset();
        assert!(!item.dismissed);
        assert_eq!(item.offset_dp, 0.0);

        let mut state = UiState::default();
        assert!(state.pal.is_dark);
        state.toggle_dark_mode();
        assert!(!state.pal.is_dark);
        state.toggle_motion_scheme();
        assert!(!state.theme.expressive_motion);
    }
}
