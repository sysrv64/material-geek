# material-geek

A Material 3 Expressive design system and GPU renderer in Rust for Android and Linux desktop.

Built from scratch in Rust as a headless core (`mg-tokens`, `mg-motion`, `mg-theme`, `mg-components`) paired with a real-time `wgpu` + `glyphon` GPU renderer (`mg-render`) and interactive 41-component catalog (`mg-android`).

Verified baseline: Jetpack Compose `material3:1.5.0-alpha` (`Compose-Material-3-Expressive-Catalog`) and Flutter `material_3_expressive` (`material_shapes.dart`, `loading_indicator.dart`, `wavy_progress_indicator.dart`).

## Status: Complete Material 3 Expressive GPU Port

| Layer | Status |
|---|---|
| `mg-tokens`: 45 color roles, 30 type styles, 10 corner token tiers + **35 Material 3 Expressive `RoundedPolygon` shapes & cubic Bézier `Morph`**, 6 elevations, easings/durations/springs | DONE |
| `mg-motion`: Dynamic Motion with analytic springs + cubic-bezier + expressive/standard `MotionScheme` + zero-jump `retarget` | DONE, covered by physics tests |
| `mg-theme`: light/dark x expressive/standard | DONE |
| `mg-components`: 41 of 41 M3 Expressive components with spec constants and Props/State | DONE |
| `mg-render`: Real-time `wgpu` WGSL SDF renderer (`RoundedBox`, 64-segment `M3ShapeMorph` across all 35 shapes, `WavyLinear` sinusoidal wave, `WavyCircularArc`) with derivative (`fwidth`) anti-aliasing, premultiplied alpha blending, pixel-accurate clipping, and `glyphon`/`cosmic-text` Roboto + Material Symbols rendering | DONE |
| `mg-android`: Interactive 41-component Material 3 Expressive gallery (`LoadingIndicator` 7-shape morph, `LinearWavyProgressIndicator`, `CircularWavyProgressIndicator`, `HorizontalMultiBrowseCarousel` / `CenteredHeroCarousel`, connected `ButtonGroup` spring physics, `SplitButton`, `FabMenu`, `WavySlider`, etc.) | DONE |

## Key Architecture Highlights

- **35 Official Material 3 Expressive Shapes (`MaterialShape`)**: Exact port of `RoundedPolygon`, `CornerRounding`, `Cubic`, and `Morph` from `material_shapes.dart` / `androidx.graphics.shapes` (`Circle`, `Square`, `Slanted`, `Arch`, `Fan`, `Arrow`, `SemiCircle`, `Oval`, `Pill`, `Triangle`, `Diamond`, `ClamShell`, `Pentagon`, `Gem`, `Sunny`, `VerySunny`, `Cookie4Sided`, `Cookie6Sided`, `Cookie7Sided`, `Cookie9Sided`, `Cookie12Sided`, `Ghostish`, `Clover4Leaf`, `Clover8Leaf`, `Burst`, `SoftBurst`, `Boom`, `SoftBoom`, `Flower`, `Puffy`, `PuffyDiamond`, `PixelCircle`, `PixelTriangle`, `Bun`, `Heart`), uploaded to a GPU uniform contour bank and evaluated per-pixel in `shape.wgsl`.
- **Authentic `LoadingIndicator` & `WavyProgressIndicator`**:
  - `LoadingIndicator` and `ContainedLoadingIndicator` morph smoothly on the GPU across the 7 official M3E shapes (`SoftBurst -> Cookie9Sided -> Pentagon -> Pill -> Sunny -> Cookie4Sided -> Oval`) with continuous + step rotation.
  - `LinearWavyProgressIndicator` and `CircularWavyProgressIndicator` (determinate & indeterminate) evaluate analytical sinusoidal wave SDFs in `shape.wgsl` with animated phase, amplitude tapering, `4dp` track gaps, and stop indicators.
- **Crisp Anti-Aliased SDF & Premultiplied Alpha Pipeline**: Derivative-based `fwidth(d)` 1px edge smoothing, inset stroke SDFs, premultiplied linear sRGB output (`col.rgb * col.a`), and pixel-space clip stack (`push_clip` / `pop_clip`).
- **Fixed `cosmic-text` / `glyphon` Text & Icon Engine**: Unbounded layout width (`16384px`) preventing single-word truncation on spaces, occurrence-indexed glyph metadata preventing depth collisions on repeated strings/icons, and non-occluding glyph depth testing.

## Quick start

```bash
cargo test --workspace
cargo run -p mg-demo
```

## Workspace

- `crates/mg-tokens`: Color, typography, elevation, motion, and 35 M3 Expressive `RoundedPolygon` + `Morph` shapes
- `crates/mg-motion`: Analytic spring solver (`DynamicSpring`) and cubic-bezier easing
- `crates/mg-theme`: Scheme and motion binding
- `crates/mg-components`: Props/State and M3E geometry constants for all 41 components
- `crates/mg-demo`: Headless core demo binary
- `crates/mg-render`: Universal `wgpu` 2D SDF + `glyphon` text/icon renderer
- `crates/mg-android`: Android (`NativeActivity`) and Linux desktop (`run_desktop`) interactive 41-component M3 Expressive gallery

## License

Apache-2.0.
