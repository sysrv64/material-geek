struct Screen {
    size: vec2f,
    _pad: vec2f,
};

struct ShapeBank {
    pts: array<vec4f, 1120>,
};

@group(0) @binding(0) var<uniform> screen: Screen;
@group(0) @binding(1) var<uniform> shape_bank: ShapeBank;

struct VIn {
    @location(0) rect: vec4f,
    @location(1) fill_col: vec4f,
    @location(2) stroke_col: vec4f,
    @location(3) radii: vec4f,
    @location(4) stroke_clip: vec4f, // clip_x, clip_y, stroke_w, rotation_or_phase
    @location(5) clip_size: vec2f,   // clip_w, clip_h
    @location(6) depth: vec2f,       // depth, kind (0=box, 1=arc/wavy_arc, 2=wavy_line, 3=m3_polygon)
};

struct VOut {
    @builtin(position) pos: vec4f,
    @location(0) local_pos: vec2f,
    @location(1) half_size: vec2f,
    @location(2) radii: vec4f,
    @location(3) fill_col: vec4f,
    @location(4) stroke_col: vec4f,
    @location(5) style_params: vec4f, // stroke_w, rotation_or_phase, kind, 0.0
    @location(6) clip: vec4f,
};

const AA_PADDING: f32 = 3.0;
const TAU: f32 = 6.283185307179586;

fn rotate(p: vec2f, angle: f32) -> vec2f {
    let s = sin(angle);
    let c = cos(angle);
    return vec2f(p.x * c - p.y * s, p.x * s + p.y * c);
}

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

    let kind = i32(round(in.depth.y));
    let diag = length(vec2f(half_w, half_h));
    // For kind == 3 (M3 polygon), expand square quad by diagonal when rotated.
    // For kind == 0 (rounded box / stroke_line), we rotate the quad vertices directly in screen space!
    let poly_rot_pad = select(0.0, diag - min(half_w, half_h), kind == 3 && in.stroke_clip.w != 0.0);
    let wave_extra = select(0.0, abs(in.radii.x) + in.radii.w * 0.5, kind == 2);
    let pad = AA_PADDING + in.stroke_clip.z * 0.5 + poly_rot_pad + wave_extra;
    let padded_half = vec2f(half_w + pad, half_h + pad);

    let corner = (vec2f(cx, cy) - vec2f(0.5)) * 2.0;
    let local = corner * padded_half;
    let world_offset = select(local, rotate(local, in.stroke_clip.w), kind == 0 && in.stroke_clip.w != 0.0);
    let px = center + world_offset;
    let ndc = vec2f(px.x / screen.size.x * 2.0 - 1.0, 1.0 - px.y / screen.size.y * 2.0);

    var out: VOut;
    out.pos = vec4f(ndc, in.depth.x, 1.0);
    out.local_pos = local;
    out.half_size = vec2f(half_w, half_h);
    out.radii = in.radii;
    out.fill_col = in.fill_col;
    out.stroke_col = in.stroke_col;
    out.style_params = vec4f(in.stroke_clip.z, in.stroke_clip.w, in.depth.y, 0.0);
    out.clip = vec4f(in.stroke_clip.xy, in.clip_size);
    return out;
}

fn sd_segment(p: vec2f, a: vec2f, b: vec2f) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-6), 0.0, 1.0);
    return length(pa - ba * h);
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

fn sd_wavy_arc(p: vec2f, b: vec2f, params: vec4f, stroke_w: f32, phase: f32) -> f32 {
    let start_ang = params.x;
    let sweep = clamp(params.y, 0.0, TAU);
    let wave_amp = params.z;
    let wave_cycles = params.w;
    let half_th = max(stroke_w * 0.5, 0.5);
    if (sweep <= 0.001) {
        return 1e5;
    }
    let base_r = max(min(b.x, b.y) - half_th - abs(wave_amp), 1.0);
    let ang = atan2(p.y, p.x);
    var rel = ang - start_ang;
    rel = rel - floor(rel / TAU) * TAU;

    let is_partial = sweep < TAU - 0.01;
    let taper_span = max(min(0.28, sweep * 0.45), 0.001);
    let env = smoothstep(0.0, taper_span, rel) * smoothstep(0.0, taper_span, sweep - rel);
    let eff_amp = select(wave_amp, wave_amp * env, is_partial);

    let arg = phase + rel * wave_cycles;
    let r_target = base_r + eff_amp * sin(arg);
    let dr = eff_amp * wave_cycles * cos(arg) / base_r;
    let grad_corr = sqrt(1.0 + dr * dr);
    let d_ring = (abs(length(p) - r_target) / grad_corr) - half_th;

    if (!is_partial) {
        return d_ring;
    }
    let p0 = vec2f(cos(start_ang), sin(start_ang)) * base_r;
    let end_ang = start_ang + sweep;
    let p1 = vec2f(cos(end_ang), sin(end_ang)) * base_r;
    let d_caps = min(length(p - p0), length(p - p1)) - half_th;
    if (rel <= sweep) {
        return min(d_ring, d_caps);
    }
    return d_caps;
}

fn wave_y_at(x: f32, half_span: f32, amp: f32, k: f32, phase: f32, taper_w: f32) -> f32 {
    let dist_edge = min(x + half_span, half_span - x);
    let env = smoothstep(0.0, taper_w, max(dist_edge, 0.0));
    return amp * env * sin(phase + (x + half_span) * k);
}

fn sd_wavy_line(p: vec2f, b: vec2f, params: vec4f) -> f32 {
    let amp = params.x;
    let wavelength = max(params.y, 2.0);
    let phase = params.z;
    let half_th = max(params.w * 0.5, 0.5);
    let half_span = max(b.x - half_th, 0.0);
    let k = TAU / wavelength;
    let taper_w = max(min(wavelength * 0.25, half_span * 0.45), 0.5);
    let xc = clamp(p.x, -half_span, half_span);
    let step = min(wavelength * 0.08, 1.8);

    var min_d = 1e5;
    var prev_x = clamp(xc - 2.0 * step, -half_span, half_span);
    var prev_pt = vec2f(prev_x, wave_y_at(prev_x, half_span, amp, k, phase, taper_w));
    for (var s = -1; s <= 2; s = s + 1) {
        let cur_x = clamp(xc + f32(s) * step, -half_span, half_span);
        let cur_pt = vec2f(cur_x, wave_y_at(cur_x, half_span, amp, k, phase, taper_w));
        min_d = min(min_d, sd_segment(p, prev_pt, cur_pt));
        prev_pt = cur_pt;
    }
    return min_d - half_th;
}

fn get_shape_vertex(shape_id: u32, vert_idx: u32) -> vec2f {
    let pair = shape_bank.pts[shape_id * 32u + (vert_idx >> 1u)];
    return select(pair.xy, pair.zw, (vert_idx & 1u) == 1u);
}

fn sample_morphed_vertex(shape_a: u32, shape_b: u32, t: f32, vert_idx: u32) -> vec2f {
    let va = get_shape_vertex(shape_a, vert_idx);
    if (shape_a == shape_b || t <= 0.0001) {
        return va;
    }
    let vb = get_shape_vertex(shape_b, vert_idx);
    return mix(va, vb, t);
}

fn sd_m3_polygon(p: vec2f, b: vec2f, params: vec4f) -> f32 {
    let shape_a = min(u32(max(params.x, 0.0)), 34u);
    let shape_b = min(u32(max(params.y, 0.0)), 34u);
    let morph_t = clamp(params.z, 0.0, 1.0);
    let scale_factor = max(params.w, 0.05);
    let scale = min(b.x, b.y) * 2.0 * scale_factor;
    let q = p / max(scale, 1.0);

    if (dot(q, q) > 0.72) {
        return (length(q) - 0.5) * scale;
    }

    var prev = sample_morphed_vertex(shape_a, shape_b, morph_t, 63u);
    let v0 = sample_morphed_vertex(shape_a, shape_b, morph_t, 0u);
    var d2 = dot(q - v0, q - v0);
    var s = 1.0;

    for (var i = 0u; i < 64u; i = i + 1u) {
        let curr = sample_morphed_vertex(shape_a, shape_b, morph_t, i);
        let e = prev - curr;
        let w = q - curr;
        let h = clamp(dot(w, e) / max(dot(e, e), 1e-8), 0.0, 1.0);
        let diff = w - e * h;
        d2 = min(d2, dot(diff, diff));

        let c1 = q.y >= curr.y;
        let c2 = q.y < prev.y;
        let c3 = (e.x * w.y) > (e.y * w.x);
        if ((c1 && c2 && c3) || (!c1 && !c2 && !c3)) {
            s = -s;
        }
        prev = curr;
    }
    return s * sqrt(d2) * scale;
}

fn srgb_to_linear(c: f32) -> f32 {
    return select(c / 12.92, pow((c + 0.055) / 1.055, 2.4), c > 0.04045);
}

@fragment
fn fs(in: VOut) -> @location(0) vec4f {
    let stroke_w = in.style_params.x;
    let rot_or_phase = in.style_params.y;
    let kind = i32(round(in.style_params.z));

    let p = select(in.local_pos, rotate(in.local_pos, -rot_or_phase), kind == 3);
    var d = 0.0;
    if (kind == 1) {
        d = sd_wavy_arc(p, in.half_size, in.radii, stroke_w, rot_or_phase);
    } else if (kind == 2) {
        d = sd_wavy_line(p, in.half_size, in.radii);
    } else if (kind == 3) {
        d = sd_m3_polygon(p, in.half_size, in.radii);
    } else {
        d = sd_rounded_box(p, in.half_size, in.radii);
    }

    let fw = clamp(fwidth(d), 0.65, 2.0);
    let aa = fw * 0.55;
    let outer_a = 1.0 - smoothstep(-aa, aa, d);
    let fill_a = outer_a;
    var stroke_a = 0.0;
    if (kind != 1 && kind != 2 && stroke_w > 0.0) {
        let inner_a = smoothstep(-stroke_w - aa, -stroke_w + aa, d);
        stroke_a = outer_a * inner_a;
    }

    let fill_alpha = in.fill_col.a * fill_a;
    let fill_lin = vec4f(
        srgb_to_linear(in.fill_col.r) * fill_alpha,
        srgb_to_linear(in.fill_col.g) * fill_alpha,
        srgb_to_linear(in.fill_col.b) * fill_alpha,
        fill_alpha,
    );
    let stroke_alpha = in.stroke_col.a * stroke_a;
    let stroke_lin = vec4f(
        srgb_to_linear(in.stroke_col.r) * stroke_alpha,
        srgb_to_linear(in.stroke_col.g) * stroke_alpha,
        srgb_to_linear(in.stroke_col.b) * stroke_alpha,
        stroke_alpha,
    );
    var out = stroke_lin + fill_lin * (1.0 - stroke_lin.a);

    if (in.clip.z > 0.0 && in.clip.w > 0.0) {
        let px = in.pos.xy;
        let mx = smoothstep(in.clip.x - 0.5, in.clip.x + 0.5, px.x)
            * (1.0 - smoothstep(in.clip.x + in.clip.z - 0.5, in.clip.x + in.clip.z + 0.5, px.x));
        let my = smoothstep(in.clip.y - 0.5, in.clip.y + 0.5, px.y)
            * (1.0 - smoothstep(in.clip.y + in.clip.w - 0.5, in.clip.y + in.clip.w + 0.5, px.y));
        out = out * (mx * my);
    }

    if (out.a <= 0.001) {
        discard;
    }
    return out;
}
