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
| `mg-android`: winit plus wgpu shell rendering the dark expressive theme with a spring-driven indicator, packaged by `cargo-apk2` | DONE, debug and signed release APKs built by CI |
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

Package `dev.geek.material`, `NativeActivity`, single `arm64-v8a` target. The shell renders the dark expressive surface color with a primary-colored circle that bounces on the expressive default spatial spring, retargeting at each settled end. The same scene is reusable on Linux desktop through the public `mg_android::run_desktop` entry.

CI builds two artifacts on every `main` push: `material-geek-apk` (debug, auto-generated debug key) and `material-geek-apk-release` (release, signed with the repo keystore and verified with `apksigner verify --print-certs`). Release signing keys live only in GitHub Secrets (`MATERIAL_GEEK_KEYSTORE_BASE64`, `MATERIAL_GEEK_KEYSTORE_PASSWORD`, plus reserved `MATERIAL_GEEK_KEY_ALIAS` and `MATERIAL_GEEK_KEY_PASSWORD`); no key material is committed.

On-device logs use the `material-geek` tag with a panic hook that forwards to logcat:

```bash
adb logcat -s material-geek
```

Init failures (surface, adapter, device) are logged instead of panicking, the adapter request falls back from high-performance to low-power with fallback adapter, and the device is requested with the adapter's own limits rather than desktop defaults.

Android lifecycle notes: the activity is `singleTask`, renderer init never kills the event loop (failures retry on the next resume), and close requests are ignored on Android so the loop survives. winit allows exactly one `EventLoop` per process, so if the system delivers a second `android_main` to a reused process the shell logs it and restarts the process for a clean slate instead of dying silently.

## Dependencies

Core workspace: single runtime dependency `serde 1.0.229`. No dependency on egui, iced, or dioxus anywhere. The `mg-android` shell crate alone uses `winit 0.30`, `wgpu 30`, `pollster 1`, `log 0.4`, and `android_logger 0.15`, all verified as latest stable releases on crates.io.

## License

Apache-2.0. Material Design specs belong to Google; this crate is an independent Rust port, not a copy of the Compose sources.
