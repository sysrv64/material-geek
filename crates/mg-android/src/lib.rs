use android_activity::{AndroidApp, MainEvent, PollEvent};
use mg_motion::{DynamicSpring, MotionScheme, Tempo, Track};
use mg_theme::GeekTheme;
use std::time::Duration;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    run_headless_self_test(&app);
    let mut finished = false;
    while !finished {
        app.poll_events(Some(Duration::from_millis(50)), |event| {
            if matches!(event, PollEvent::Main(MainEvent::Destroy)) {
                finished = true;
            }
        });
    }
}

fn run_headless_self_test(app: &AndroidApp) {
    let theme = GeekTheme::dark_expressive();
    let scheme = MotionScheme::expressive();
    let spec = scheme.spec(Tempo::Default, Track::Spatial);
    let mut spring = DynamicSpring::new(0.0, 1.0, spec);
    for _ in 0..240 {
        spring.step(1.0 / 120.0);
    }
    let payload = format!(
        "dark={} motion={:?} x={:.4} settled={}\n",
        theme.scheme.is_dark,
        theme.scheme_kind(),
        spring.x,
        spring.settled()
    );
    if let Some(dir) = app.internal_data_path() {
        let _ = std::fs::write(dir.join("material-geek-status.txt"), payload);
    }
}
