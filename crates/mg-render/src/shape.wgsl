struct Screen {
    size: vec2f,
};
@group(0) @binding(0) var<uniform> screen: Screen;

struct VIn {
    @location(0) rect: vec4f,
    @location(1) fill_col: vec4f,
    @location(2) stroke_col: vec4f,
    @location(3) radii: vec4f,
    @location(4) stroke_clip: vec4f,
    @location(5) clip_size: vec2f,
    @location(6) depth: vec2f,
};

struct VOut {
    @builtin(position) pos: vec4f,
    @location(0) corner: vec2f,
    @location(1) half_size: vec2f,
    @location(2) radii: vec4f,
    @location(3) fill_col: vec4f,
    @location(4) stroke_col: vec4f,
    @location(5) stroke_w: f32,
    @location(6) clip: vec4f,
};

@vertex
fn vs(@builtin(vertex_index) i: u32, in: VIn) -> VOut {
    var cx = 0.0;
    var cy = 0.0;
    if (i == 1u || i == 2u || i == 5u) {
        cx = 1.0;
    }
    if (i == 2u || i == 3u || i == 5u) {
        cy = 1.0;
    }
    let px = vec2f(in.rect.x + cx * in.rect.z, in.rect.y + cy * in.rect.w);
    let ndc = vec2f(px.x / screen.size.x * 2.0 - 1.0, 1.0 - px.y / screen.size.y * 2.0);
    var out: VOut;
    out.pos = vec4f(ndc, 0.0, 1.0);
    out.corner = vec2f(cx, cy);
    out.half_size = in.rect.zw * 0.5;
    out.radii = in.radii;
    out.fill_col = in.fill_col;
    out.stroke_col = in.stroke_col;
    out.stroke_w = in.stroke_clip.z;
    out.clip = vec4f(in.stroke_clip.xy, in.clip_size);
    out.pos = vec4f(ndc, in.depth.x, 1.0);
    return out;
}

fn sd_rounded_box(p: vec2f, b: vec2f, rad: vec4f) -> f32 {
    let r = select(
        select(rad.w, rad.x, p.y < 0.0),
        select(rad.z, rad.y, p.y < 0.0),
        p.x > 0.0,
    );
    let rc = min(r, min(b.x, b.y));
    let q = abs(p) - b + vec2f(rc);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2f(0.0))) - rc;
}

fn srgb_to_linear(c: f32) -> f32 {
    return select(c / 12.92, pow((c + 0.055) / 1.055, 2.4), c > 0.04045);
}

@fragment
fn fs(in: VOut) -> @location(0) vec4f {
    let local = (in.corner - vec2f(0.5)) * in.half_size * 2.0;
    let d = sd_rounded_box(local, in.half_size, in.radii);
    let aa = max(fwidth(d), 0.0001);
    let fill_a = 1.0 - smoothstep(-aa, aa, d);
    var stroke_a = 1.0 - smoothstep(-aa, aa, abs(d) - in.stroke_w * 0.5);
    if (in.stroke_w <= 0.0) {
        stroke_a = 0.0;
    }
    var fill = vec4f(
        srgb_to_linear(in.fill_col.r),
        srgb_to_linear(in.fill_col.g),
        srgb_to_linear(in.fill_col.b),
        in.fill_col.a,
    );
    var stroke = vec4f(
        srgb_to_linear(in.stroke_col.r),
        srgb_to_linear(in.stroke_col.g),
        srgb_to_linear(in.stroke_col.b),
        in.stroke_col.a,
    );
    fill = fill * vec4f(vec3f(1.0), fill_a);
    stroke = stroke * vec4f(vec3f(1.0), stroke_a);
    let out = stroke + fill * (1.0 - stroke.a);
    if (out.a <= 0.0) {
        discard;
    }
    var alpha = clamp(out.a, 0.0, 1.0);
    if (in.clip.z > 0.0) {
        let px = vec2f(
            (in.pos.x + 1.0) * 0.5 * screen.size.x,
            (1.0 - in.pos.y) * 0.5 * screen.size.y,
        );
        let mx = smoothstep(in.clip.x - 0.5, in.clip.x + 0.5, px.x)
            * (1.0 - smoothstep(in.clip.x + in.clip.z - 0.5, in.clip.x + in.clip.z + 0.5, px.x));
        let my = smoothstep(in.clip.y - 0.5, in.clip.y + 0.5, px.y)
            * (1.0 - smoothstep(in.clip.y + in.clip.w - 0.5, in.clip.y + in.clip.w + 0.5, px.y));
        alpha = alpha * mx * my;
    }
    let rgb = select(vec3f(0.0), out.rgb / vec3f(alpha), alpha > 0.0);
    return vec4f(rgb, alpha);
}
