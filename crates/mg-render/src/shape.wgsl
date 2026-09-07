struct Screen {
    size: vec2f,
};
@group(0) @binding(0) var<uniform> screen: Screen;

struct VIn {
    @location(0) rect: vec4f,
    @location(1) fill_col: vec4f,
    @location(2) stroke_col: vec4f,
    @location(3) radii: vec4f,
    @location(4) stroke_clip: vec4f, // clip_x, clip_y, stroke_w, rotation
    @location(5) clip_size: vec2f, // clip_w, clip_h
    @location(6) depth: vec2f,
};

struct VOut {
    @builtin(position) pos: vec4f,
    @location(0) local_pos: vec2f,
    @location(1) half_size: vec2f,
    @location(2) radii: vec4f,
    @location(3) fill_col: vec4f,
    @location(4) stroke_col: vec4f,
    @location(5) stroke_w: f32,
    @location(6) rotation: f32,
    @location(7) clip: vec4f,
};

const AA_PADDING: f32 = 3.0;

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
    let half_w = in.rect.z * 0.5;
    let half_h = in.rect.w * 0.5;
    let center = in.rect.xy + vec2f(half_w, half_h);

    // Expand the quad outward so the SDF has room for anti-aliased edges
    // and strokes; without this the rasterizer clips the smooth falloff
    // and rounded corners come out chopped.
    let diag = length(vec2f(half_w, half_h));
    let rot_padding = select(0.0, diag - max(half_w, half_h), in.stroke_clip.w != 0.0);
    let pad = AA_PADDING + in.stroke_clip.z * 0.5 + rot_padding;
    let padded_half = vec2f(half_w + pad, half_h + pad);

    let corner = (vec2f(cx, cy) - vec2f(0.5)) * 2.0;
    let local = corner * padded_half;
    let px = center + local;
    let ndc = vec2f(px.x / screen.size.x * 2.0 - 1.0, 1.0 - px.y / screen.size.y * 2.0);

    var out: VOut;
    out.pos = vec4f(ndc, in.depth.x, 1.0);
    out.local_pos = local;
    out.half_size = vec2f(half_w, half_h);
    out.radii = in.radii;
    out.fill_col = in.fill_col;
    out.stroke_col = in.stroke_col;
    out.stroke_w = in.stroke_clip.z;
    out.rotation = in.stroke_clip.w;
    out.clip = vec4f(in.stroke_clip.xy, in.clip_size);
    return out;
}

fn rotate(p: vec2f, angle: f32) -> vec2f {
    let s = sin(angle);
    let c = cos(angle);
    return vec2f(p.x * c - p.y * s, p.x * s + p.y * c);
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
    // Rotation happens inside the SDF so capsules and bars render natively
    // at any angle without vertex-level rotation.
    let p = rotate(in.local_pos, -in.rotation);
    let d = sd_rounded_box(p, in.half_size, in.radii);
    let aa = max(fwidth(d), 0.7);
    let fill_a = 1.0 - smoothstep(-aa, aa, d);
    var stroke_a = 0.0;
    if (in.stroke_w > 0.0) {
        stroke_a = 1.0 - smoothstep(-aa, aa, abs(d) - in.stroke_w * 0.5);
    }
    let fill_lin = vec4f(
        srgb_to_linear(in.fill_col.r),
        srgb_to_linear(in.fill_col.g),
        srgb_to_linear(in.fill_col.b),
        in.fill_col.a * fill_a,
    );
    let stroke_lin = vec4f(
        srgb_to_linear(in.stroke_col.r),
        srgb_to_linear(in.stroke_col.g),
        srgb_to_linear(in.stroke_col.b),
        in.stroke_col.a * stroke_a,
    );
    var out = stroke_lin + fill_lin * (1.0 - stroke_lin.a);
    if (out.a <= 0.001) {
        discard;
    }
    if (in.clip.z > 0.0) {
        let px = vec2f(
            (in.pos.x + 1.0) * 0.5 * screen.size.x,
            (1.0 - in.pos.y) * 0.5 * screen.size.y,
        );
        let mx = smoothstep(in.clip.x - 0.5, in.clip.x + 0.5, px.x)
            * (1.0 - smoothstep(in.clip.x + in.clip.z - 0.5, in.clip.x + in.clip.z + 0.5, px.x));
        let my = smoothstep(in.clip.y - 0.5, in.clip.y + 0.5, px.y)
            * (1.0 - smoothstep(in.clip.y + in.clip.w - 0.5, in.clip.y + in.clip.w + 0.5, px.y));
        out = out * (mx * my);
    }
    return out;
}
