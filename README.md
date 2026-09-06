# material-geek

A Material 3 Expressive idea for developers and geeks: a universal (Android, Linux desktop) design system in Rust.

An alternative to material3-multiplatform, built from scratch in Rust as a headless core (tokens + state) with thin render adapters.

Verified baseline: Jetpack `material3:1.5.0-alpha25` (stable `1.4.0`) + JetBrains CMP `1.12.0` (`material3:1.12.0-alpha03` = Jetpack `1.5.0-alpha22`).

## Status: v0.1.0, core only, not a pixel port

| Layer | Status |
|---|---|
| `mg-tokens`: 45 color roles, 30 type styles, 10 shapes, 6 elevations, easings/durations/springs | DONE |
| `mg-motion`: Dynamic Motion with analytic springs + cubic-bezier + expressive/standard MotionScheme + retarget | DONE, covered by physics tests |
| `mg-theme`: light/dark x expressive/standard | DONE |
| `mg-components`: 41 of 41 components as headless Props/State | DONE, API surface |
| `mg-android`: NativeActivity shell (`android-activity 0.6`) running the headless core on device, packaged by `cargo-apk2` | DONE, APK built by CI |
| Render adapter with real pixels (desktop + Android surface) | TODO in v0.2 |
| Dynamic-color HCT, shape morph, text layout | TODO in v0.2 |

Scope note: "all elements ported" in v0.1 means state specs for all 41 components plus exact tokens plus motion physics. Pixel rendering of each component is v0.2 work.

## Geek UX

- Everything is inspectable: the theme serializes with `serde`, a spring can be stopped at any frame to read `(x, v)`.
- Springs instead of tweens for interruptible gestures: target changes mid-flight with no jumps (`DynamicSpring::retarget`).
- `MotionScheme::expressive()` vs `standard()` switches in one line; spatial tracks (damping below 1.0, overshoot) are separated from effects tracks (damping exactly 1.0, strictly no overshoot).
- `ComponentId::ALL` is a registry of 41 components, guarded against drift by a test.

## Quick start

```bash
cargo test --workspace
cargo run -p mg-demo --bin geek_preview
```

```rust
use mg_motion::{DynamicSpring, MotionScheme, Tempo, Track};
use mg_tokens::motion_tokens::schemes;

let scheme = MotionScheme::expressive();
let spec = scheme.spec(Tempo::Default, Track::Spatial);
assert_eq!(spec, schemes::EXPRESSIVE_DEFAULT_SPATIAL);

let mut s = DynamicSpring::new(0.0, 1.0, spec);
s.step(1.0 / 120.0);
s.retarget(0.0);
```

## Workspace

- `crates/mg-tokens`: pure data, zero GUI dependencies
- `crates/mg-motion`: physics with analytic solution, not Euler integration
- `crates/mg-theme`: scheme plus motion binding
- `crates/mg-components`: headless Props/State for 41 components
- `crates/mg-demo`: headless core demo binary
- `crates/mg-android`: Android shell, excluded from the host workspace, built only for the `aarch64-linux-android` target in CI

## Android APK

Package `dev.geek.material`, `NativeActivity`, single `arm64-v8a` target. CI installs Android NDK r29 (`29.0.14206865`), builds with `cargo apk2 build`, and uploads the APK as the `material-geek-apk` artifact. On launch the shell runs the headless self-test (theme resolve plus 240 spring steps) and writes `material-geek-status.txt` to the app internal data directory. Pixel rendering arrives in v0.2.

## Dependencies

Single runtime dependency: `serde 1.0.229`, verified as the latest stable release on crates.io. No dependency on egui, iced, dioxus, winit, or wgpu in v0.1. Render-adapter crates (winit, wgpu, layout, text) are scheduled for v0.2 with pinned versions resolved at that time.

## License

Apache-2.0. Material Design specs belong to Google; this crate is an independent Rust port, not a copy of the Compose sources.
