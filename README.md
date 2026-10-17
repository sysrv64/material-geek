# material-geek

A full Material 3 Expressive GPU UI Framework and design system in Rust for Android (`NativeActivity`) and Linux/desktop (`winit` + `wgpu`).

Built from scratch in Rust with a modular architecture:
- **`mg-tokens`**: 45 M3 color roles, 30 typography tokens, 10 corner tiers, and **35 Material 3 Expressive `RoundedPolygon` shapes & cubic Bézier `Morph`**.
- **`mg-motion`**: Analytic second-order spring solver (`DynamicSpring`), cubic-bezier easing, and `Expressive` / `Standard` `MotionScheme` with zero-jump retargeting.
- **`mg-theme`**: Live `Light` / `Dark` color scheme x `Expressive` / `Standard` motion scheme binding (`GeekTheme`).
- **`mg-render`**: Real-time `wgpu` WGSL SDF renderer (`RoundedBox`, 64-segment `M3ShapeMorph` across all 35 shapes, `WavyLinear` sinusoidal wave, `WavyCircularArc`) with `fwidth` anti-aliasing, premultiplied alpha blending, pixel-accurate clip stack, and `glyphon` / `cosmic-text` Roboto + Material Symbols rendering.
- **`mg-components`**: **Complete Material 3 Expressive UI Framework** providing `UiState`, `UiContext`, layout containers (`ColumnLayout`, `RowLayout`), focus & keyboard management, built-in docked M3E `SoftKeyboard` (`ABC`, `123 / Date`, `#+=$`), gesture recognition (`Tap`, `ImmediateDragX`, `SwipeX`, `PullY`, `Dial`), stateful controllers (`DatePickerState`, `TimePickerState`, `TextFieldProps`, `SwipeDismissItem`, `PullRefreshState`), and reusable GPU widget methods for all 41 M3 Expressive components.
- **`mg-android`**: Thin platform runner (`winit` window/touch/mouse/keyboard/IME event bridge + `wgpu` surface management) hosting the interactive 41-component catalog built on `mg-components`.
- **`mg-desktop`**: Minimal Linux/desktop/Windows/macOS window runner (`mg-geek` binary) reusing the platform-neutral `mg-android` gallery runner.
- **`mg-studio`**: Separate **PC application** (`mg-studio` binary) — the 41-component catalog rendered at desktop pixel density in the M3 adaptive list-detail layout (clickable navigation pane + scrollable detail pane), hardware keyboard input, no phone chrome.

Verified against Jetpack Compose `material3:1.5.0-alpha` (`Compose-Material-3-Expressive-Catalog`) and Flutter `material_3_expressive`.

## Framework Usage Example

```rust
use mg_components::{
    ButtonProps, ColumnLayout, DatePickerState, Rect, SliderProps, TextFieldProps, UiContext,
    UiState,
};
use mg_render::UiRenderer;

pub struct MyAppState {
    pub ui: UiState,
    pub email: TextFieldProps,
    pub volume: SliderProps,
    pub date_picker: DatePickerState,
}

pub fn draw_frame(app: &mut MyAppState, renderer: &mut UiRenderer, width: f32, height: f32) {
    let mut ctx = UiContext::begin(renderer, &mut app.ui, width, height);
    let mut col = ColumnLayout::new(16.0, 24.0, ctx.width - 32.0, 16.0);

    // Interactive TextField with hardware keyboard + built-in M3E touch keyboard
    let tf_rect = col.next(56.0);
    ctx.text_field("email_input", tf_rect, &mut app.email);

    // Expressive WavySlider with live drag & floating value indicator bubble
    let slider_rect = col.next(44.0);
    ctx.slider("volume_slider", slider_rect, &mut app.volume);

    // Expressive shape-morphing Filled Button
    let btn_rect = col.next(40.0);
    if ctx
        .button_rect("submit_btn", btn_rect, &ButtonProps::filled("Save"))
        .clicked
    {
        app.ui.toggle_dark_mode();
    }

    // Full M3 DatePicker with Calendar grid, Year picker, and Pencil ("edit") numeric DateInput mode
    ctx.date_picker("booking_date", col.x, col.y, col.w, &mut app.date_picker);

    ctx.draw_ripples();
    ctx.draw_soft_keyboard();
}
```

## Key Interactive Capabilities

- **M3 `DatePicker` & `DateInput`**: Full Gregorian calendar grid, `<` / `>` month navigation, interactive Year picker grid (`2020..=2034`), and **Pencil (`edit` <-> `calendar_today`) mode toggle** switching into M3 numeric date entry (`MM/DD/YYYY`, `YYYY-MM-DD`, `DD.MM.YYYY`, `MMDDYYYY`) with live validation and quick presets.
- **Full Text Input (`TextField`, `OutlinedTextField`, `SearchBar`)**: Hardware keyboard (`WindowEvent::KeyboardInput` + `Ime::Commit`) and built-in docked M3 Expressive **SoftKeyboard** (`ABC`, `123 / Date`, `#+=$`) for Android `NativeActivity` and touchscreens, with blinking caret, cursor movement, password visibility toggle (`visibility` / `visibility_off`), clear button, and character counter validation.
- **`SwipeToDismiss` Physics**: Horizontal drag gesture (`ZoneKind::SwipeX`) with `DynamicSpring` snapback/dismissal, dynamic background pill reveal (`Archive` / `Delete`), threshold detection, and `Undo` / `Reset` restoration.
- **`TimePicker` Dial & Numeric Input**: Drag-rotatable clock hand on 12-hour / 60-minute dial plus keyboard toggle (`keyboard` <-> `schedule`) for direct numeric Hour/Minute entry.
- **`PullToRefresh`**: Vertical pull gesture (`ZoneKind::PullY`) with `ContainedLoadingIndicator` 7-shape morphing and live feed prepending.
- **Live Theme & Motion Switching**: Instant toggle between `Dark` and `Light` M3 color schemes and `Expressive` vs `Standard` spring physics from the top app bar.

## Quick start

```bash
cargo test --workspace

# headless core demo
cargo run -p mg-demo

# Linux/desktop window (auto-scaled catalog)
cargo run -p mg-desktop

# PC app: adaptive list-detail catalog at desktop density
cargo run -p mg-studio
```

## Cross-platform (multiplatform) support

The component stack (`mg-tokens`/`mg-motion`/`mg-theme`/`mg-render`/`mg-components`) is
form-factor neutral; `UiState` exposes:

- `density`: fixed px/dp scale override (`0.0` = width-derived, phone behavior) — desktop windows run at a desktop-appropriate scale instead of the phone width scaling.
- `soft_keyboard`: disables the docked M3E soft keyboard on hardware-keyboard form factors while keeping the same `focus()`/`push_key()` text-input path.
- `top_inset_dp`: phone status-bar inset override (desktop runners set `0.0`).
- `is_paned_layout()`: M3 adaptive breakpoint (`>= 840dp`) switching the catalog between the compact single-column phone layout and the list-detail navigation-pane layout.

Android (`NativeActivity`) and phone behavior are unchanged; desktop runners
(`mg-desktop`, `mg-studio`) configure the above and add an independent `winit` app process.

## License

Apache-2.0.
