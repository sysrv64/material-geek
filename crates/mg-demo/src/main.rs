use mg_components::{
    CatalogState, ComponentId, DatePickerEntryMode, DatePickerState, TextFieldProps,
    TimePickerState, UiState,
};
use mg_motion::{DynamicSpring, MotionScheme, Tempo, Track};
use mg_theme::GeekTheme;

fn main() {
    let theme = GeekTheme::dark_expressive();
    println!(
        "scheme dark={} motion={:?}",
        theme.scheme.is_dark,
        theme.scheme_kind()
    );
    println!("components: {}", ComponentId::ALL.len());

    let mut ui_state = UiState::new(theme);
    let mut catalog = CatalogState::new();

    // Demonstrate DatePicker pencil toggle & numeric date parsing
    catalog.date_picker.toggle_entry_mode();
    assert_eq!(catalog.date_picker.entry_mode, DatePickerEntryMode::Input);
    catalog.date_picker.input_field.text = "12/31/2026".to_string();
    assert!(catalog.date_picker.sync_date_from_input());
    println!(
        "DatePicker input mode parsed: {} ({})",
        DatePickerState::format_mm_dd_yyyy(
            catalog.date_picker.year,
            catalog.date_picker.month,
            catalog.date_picker.day
        ),
        catalog.date_picker.format_headline()
    );

    // Demonstrate TextField editing & password visibility
    let mut tf = TextFieldProps::filled("Project", "Material ");
    tf.insert_str("Geek Framework");
    println!("TextField value: {} (cursor={})", tf.text, tf.cursor);

    // Demonstrate TimePicker numeric input
    let mut tp = TimePickerState::new(10, 25, true);
    tp.toggle_input_mode();
    tp.hour_field.text = "07".to_string();
    tp.minute_field.text = "45".to_string();
    tp.is_am = false;
    let _ = tp.confirm();
    println!("TimePicker confirmed: {}", tp.format_time());

    ui_state.step(1.0 / 60.0);

    let scheme = MotionScheme::expressive();
    let spatial = scheme.spec(Tempo::Default, Track::Spatial);
    let mut s = DynamicSpring::new(0.0, 1.0, spatial);
    for i in 0..5 {
        s.step(1.0 / 60.0);
        println!("t={:>4}ms x={:.4} v={:.4}", (i + 1) * 16, s.x, s.v);
    }
    s.retarget(0.0);
    println!("retarget -> 0.0, x={:.4} (no jump)", s.x);
}
