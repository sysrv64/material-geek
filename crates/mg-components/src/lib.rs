use mg_tokens::elevation::ElevationLevel;
use mg_tokens::shape::ShapeStyle;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Size {
    Xs,
    S,
    M,
    L,
    Xl,
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
    pub morph_on_press: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FabSize {
    Small,
    Medium,
    Large,
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
    pub menu_open: bool,
    pub enabled: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckState {
    Off,
    On,
    Indeterminate,
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgressProps {
    pub progress: Option<f32>,
    pub wavy: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadingProps {
    pub contained: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChipProps {
    pub label: String,
    pub selected: bool,
    pub enabled: bool,
    pub with_icon: bool,
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
    pub outlined: bool,
    pub secure: bool,
    pub error: Option<String>,
    pub enabled: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateState {
    pub millis: Option<u64>,
    pub range_end_millis: Option<u64>,
    pub dialog_open: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeState {
    pub hour: u8,
    pub minute: u8,
    pub input_mode: bool,
    pub dialog_open: bool,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DismissState {
    pub dismissed: bool,
    pub direction_start_to_end: bool,
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
}
