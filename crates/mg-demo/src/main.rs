use mg_components::ComponentId;
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

    let scheme = MotionScheme::expressive();
    let spatial = scheme.spec(Tempo::Default, Track::Spatial);
    let effects = scheme.spec(Tempo::Default, Track::Effects);
    println!(
        "spatial damping={} stiffness={}",
        spatial.damping_ratio, spatial.stiffness
    );
    println!(
        "effects damping={} stiffness={}",
        effects.damping_ratio, effects.stiffness
    );

    let mut s = DynamicSpring::new(0.0, 1.0, spatial);
    for i in 0..10 {
        s.step(1.0 / 60.0);
        println!("t={:>4}ms x={:.4} v={:.4}", (i + 1) * 16, s.x, s.v);
    }
    s.retarget(0.0);
    println!("retarget -> 0.0, x={:.4} (no jump)", s.x);
}
