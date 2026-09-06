use ab_glyph::{point, Font, FontRef, PxScale, ScaleFont};
use std::collections::HashMap;
pub mod icons;
pub use icons::{IconCmd, IconPath};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BlendState,
    Buffer, BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites, Extent3d,
    FilterMode, FragmentState, MultisampleState, Origin3d, RenderPipeline,
    RenderPipelineDescriptor, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode,
};

const REGULAR: &[u8] = include_bytes!("../assets/GoogleSansCode-Regular.ttf");
const MEDIUM: &[u8] = include_bytes!("../assets/GoogleSansCode-Medium.ttf");
const ATLAS_SIZE: u32 = 2048;

const SHADER: &str = r#"
struct Screen {
    size: vec2f,
};
@group(0) @binding(0) var<uniform> screen: Screen;
@group(1) @binding(0) var atlas_tex: texture_2d<f32>;
@group(1) @binding(1) var atlas_samp: sampler;
struct VIn {
    @location(1) rect: vec4f,
    @location(2) color: vec4f,
    @location(3) uv: vec4f,
    @location(4) misc: vec4f,
    @location(5) clip: vec4f,
};
struct VOut {
    @builtin(position) pos: vec4f,
    @location(0) corner: vec2f,
    @location(1) color: vec4f,
    @location(2) uv: vec2f,
    @location(3) uv_size: vec2f,
    @location(4) misc: vec4f,
    @location(5) half_size: vec2f,
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
    out.color = in.color;
    out.uv = in.uv.xy + vec2f(cx * in.uv.z, cy * in.uv.w);
    out.uv_size = in.uv.zw;
    out.misc = in.misc;
    out.half_size = in.rect.zw * 0.5;
    out.clip = in.clip;
    return out;
}
fn sd_box(p: vec2f, b: vec2f, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return length(max(q, vec2f(0.0))) + min(max(q.x, q.y), 0.0) - r;
}
@fragment
fn fs(in: VOut) -> @location(0) vec4f {
    let local = (in.corner - vec2f(0.5)) * in.half_size * 2.0;
    let r = min(in.misc.x, min(in.half_size.x, in.half_size.y));
    let d = sd_box(local, in.half_size, r);
    let mask = 1.0 - smoothstep(-0.75, 0.75, d);
    var col = in.color;
    if (in.misc.y > 0.5) {
        let a = textureSample(atlas_tex, atlas_samp, in.uv).r;
        col = vec4f(in.color.rgb, in.color.a * a);
    }
    var alpha = col.a * mask;
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
    return vec4f(col.rgb, alpha);
}
"#;

#[derive(Clone, Copy)]
struct GlyphKey {
    font: u8,
    id: u16,
    px: u32,
}

impl std::hash::Hash for GlyphKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u8(self.font);
        state.write_u16(self.id);
        state.write_u32(self.px);
    }
}

impl PartialEq for GlyphKey {
    fn eq(&self, other: &Self) -> bool {
        self.font == other.font && self.id == other.id && self.px == other.px
    }
}

impl Eq for GlyphKey {}

fn glyph_metrics(bounds: ab_glyph::Rect) -> (u32, u32) {
    (bounds.width().ceil() as u32, bounds.height().ceil() as u32)
}

fn glyph_uv(ox: u32, oy: u32, gw: u32, gh: u32) -> [f32; 4] {
    let s = ATLAS_SIZE as f32;
    [ox as f32 / s, oy as f32 / s, gw as f32 / s, gh as f32 / s]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStyle {
    pub px: f32,
    pub line_h: f32,
    pub medium: bool,
    pub tracking_em: f32,
}

impl TypeStyle {
    pub const fn new(px: f32, line_h: f32, medium: bool, tracking_em: f32) -> Self {
        Self {
            px,
            line_h,
            medium,
            tracking_em,
        }
    }
}

pub const M3_DISPLAY_LARGE: TypeStyle = TypeStyle::new(57.0, 64.0, false, -0.2 / 57.0);
pub const M3_DISPLAY_MEDIUM: TypeStyle = TypeStyle::new(45.0, 52.0, false, 0.0);
pub const M3_DISPLAY_SMALL: TypeStyle = TypeStyle::new(36.0, 44.0, false, 0.0);
pub const M3_HEADLINE_LARGE: TypeStyle = TypeStyle::new(32.0, 40.0, false, 0.0);
pub const M3_HEADLINE_MEDIUM: TypeStyle = TypeStyle::new(28.0, 36.0, false, 0.0);
pub const M3_HEADLINE_SMALL: TypeStyle = TypeStyle::new(24.0, 32.0, false, 0.0);
pub const M3_TITLE_LARGE: TypeStyle = TypeStyle::new(22.0, 28.0, false, 0.0);
pub const M3_TITLE_MEDIUM: TypeStyle = TypeStyle::new(16.0, 24.0, true, 0.2 / 16.0);
pub const M3_TITLE_SMALL: TypeStyle = TypeStyle::new(14.0, 20.0, true, 0.1 / 14.0);
pub const M3_BODY_LARGE: TypeStyle = TypeStyle::new(16.0, 24.0, false, 0.5 / 16.0);
pub const M3_BODY_MEDIUM: TypeStyle = TypeStyle::new(14.0, 20.0, false, 0.2 / 14.0);
pub const M3_BODY_SMALL: TypeStyle = TypeStyle::new(12.0, 16.0, false, 0.4 / 12.0);
pub const M3_LABEL_LARGE: TypeStyle = TypeStyle::new(14.0, 20.0, true, 0.1 / 14.0);
pub const M3_LABEL_MEDIUM: TypeStyle = TypeStyle::new(12.0, 16.0, true, 0.5 / 12.0);
pub const M3_LABEL_SMALL: TypeStyle = TypeStyle::new(11.0, 16.0, true, 0.5 / 11.0);

pub fn m3_text_style(t: TypeStyle, color: [f32; 4]) -> TextStyle {
    TextStyle {
        px: t.px,
        medium: t.medium,
        color,
        tracking_em: t.tracking_em,
        line_h: t.line_h,
    }
}

pub struct UiRenderer {
    pipeline: RenderPipeline,
    screen_buf: Buffer,
    screen_group: BindGroup,
    atlas_tex: wgpu::Texture,
    atlas_group: BindGroup,
    atlas_data: Vec<u8>,
    atlas_dirty: bool,
    cursor_x: u32,
    cursor_y: u32,
    row_h: u32,
    glyphs: HashMap<GlyphKey, GlyphEntry>,
    icon_cells: HashMap<u32, [f32; 4]>,
    fonts: [FontRef<'static>; 2],
    instances: Vec<f32>,
    instance_buf: Buffer,
    instance_cap: usize,
}

#[derive(Clone, Copy)]
pub struct Quad {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
    pub uv: [f32; 4],
    pub radius: f32,
    pub mode: f32,
    pub clip: [f32; 4],
}

#[derive(Clone, Copy)]
pub struct TextStyle {
    pub px: f32,
    pub medium: bool,
    pub color: [f32; 4],
    pub tracking_em: f32,
    pub line_h: f32,
}

#[derive(Clone, Copy)]
struct GlyphEntry {
    uv: [f32; 4],
    bx: f32,
    by: f32,
}

impl UiRenderer {
    pub fn new(device: &wgpu::Device, format: TextureFormat) -> Self {
        let fonts = [
            FontRef::try_from_slice(REGULAR).expect("regular font"),
            FontRef::try_from_slice(MEDIUM).expect("medium font"),
        ];
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(SHADER.into()),
        });
        let instance_attrs = [
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 1,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 16,
                shader_location: 2,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 3,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 48,
                shader_location: 4,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 64,
                shader_location: 5,
            },
        ];
        let instance_layout = VertexBufferLayout {
            array_stride: 80,
            step_mode: VertexStepMode::Instance,
            attributes: &instance_attrs,
        };
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[Some(instance_layout)],
                compilation_options: Default::default(),
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let screen_buf = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 16,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let screen_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &screen_buf,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        let atlas_tex = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let atlas_view = atlas_tex.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            ..Default::default()
        });
        let atlas_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(1),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&atlas_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });
        let instance_buf = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 80 * 256,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            screen_buf,
            screen_group,
            atlas_tex,
            atlas_group,
            atlas_data: vec![0u8; (ATLAS_SIZE * ATLAS_SIZE) as usize],
            atlas_dirty: false,
            cursor_x: 0,
            cursor_y: 0,
            row_h: 0,
            glyphs: HashMap::new(),
            icon_cells: HashMap::new(),
            fonts,
            instances: Vec::with_capacity(256 * 16),
            instance_buf,
            instance_cap: 256,
        }
    }

    pub fn set_screen(&mut self, queue: &wgpu::Queue, w: f32, h: f32) {
        let bytes = [w.to_ne_bytes(), h.to_ne_bytes()].concat();
        queue.write_buffer(&self.screen_buf, 0, &bytes);
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4], radius: f32) {
        self.push(Quad {
            x,
            y,
            w,
            h,
            color,
            uv: [0.0, 0.0, 0.0, 0.0],
            radius,
            mode: 0.0,
            clip: [0.0; 4],
        });
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, color: [f32; 4]) {
        self.push(Quad {
            x: cx - r,
            y: cy - r,
            w: r * 2.0,
            h: r * 2.0,
            color,
            uv: [0.0, 0.0, 0.0, 0.0],
            radius: r,
            mode: 0.0,
            clip: [0.0; 4],
        });
    }

    fn push(&mut self, q: Quad) {
        self.instances.extend_from_slice(&[
            q.x, q.y, q.w, q.h, q.color[0], q.color[1], q.color[2], q.color[3], q.uv[0], q.uv[1],
            q.uv[2], q.uv[3], q.radius, q.mode, 0.0, 0.0, q.clip[0], q.clip[1], q.clip[2],
            q.clip[3],
        ]);
    }

    fn rasterize(&mut self, font_idx: usize, c: char, px: f32) -> Option<GlyphEntry> {
        let font = &self.fonts[font_idx];
        let gid = font.glyph_id(c);
        let key = GlyphKey {
            font: font_idx as u8,
            id: gid.0,
            px: (px * 16.0) as u32,
        };
        if let Some(entry) = self.glyphs.get(&key) {
            return Some(*entry);
        }
        let scaled = font.as_scaled(PxScale::from(px));
        let mut glyph = scaled.scaled_glyph(c);
        glyph.position = point(0.0, 0.0);
        let outlined = scaled.outline_glyph(glyph)?;
        let bounds = outlined.px_bounds();
        let (gw, gh) = glyph_metrics(bounds);
        if self.cursor_x + gw + 2 > ATLAS_SIZE {
            self.cursor_x = 0;
            self.cursor_y += self.row_h + 2;
            self.row_h = 0;
        }
        if self.cursor_y + gh + 2 > ATLAS_SIZE {
            return None;
        }
        let ox = self.cursor_x + 1;
        let oy = self.cursor_y + 1;
        outlined.draw(|gx, gy, v| {
            let idx = ((oy + gy) * ATLAS_SIZE + ox + gx) as usize;
            if idx < self.atlas_data.len() {
                self.atlas_data[idx] = (v * 255.0) as u8;
            }
        });
        self.cursor_x += gw + 2;
        self.row_h = self.row_h.max(gh + 2);
        self.atlas_dirty = true;
        let entry = GlyphEntry {
            uv: glyph_uv(ox, oy, gw, gh),
            bx: bounds.min.x,
            by: bounds.min.y,
        };
        self.glyphs.insert(key, entry);
        Some(entry)
    }

    pub fn icon(&mut self, name: &str, x: f32, y: f32, px: f32, color: [f32; 4]) {
        let Some(idx) = icons::icon_index(name) else {
            return;
        };
        let Some(uv) = self.rasterize_icon(idx, px) else {
            return;
        };
        self.push(Quad {
            x,
            y,
            w: uv[2] * ATLAS_SIZE as f32,
            h: uv[3] * ATLAS_SIZE as f32,
            color,
            uv,
            radius: 0.0,
            mode: 1.0,
            clip: [0.0; 4],
        });
    }

    fn rasterize_icon(&mut self, idx: usize, px: f32) -> Option<[f32; 4]> {
        let key = ((idx as u32) << 12) | ((px * 4.0) as u32);
        if let Some(uv) = self.icon_cells.get(&key) {
            return Some(*uv);
        }
        let path = icons::ICONS.get(idx)?.1;
        let s = px / path.w;
        let mut edges: Vec<(f32, f32, f32, f32)> = Vec::new();
        let mut cur = (0.0f32, 0.0f32);
        let mut sub_start = cur;
        let push_seg = |edges: &mut Vec<(f32, f32, f32, f32)>, a: (f32, f32), b: (f32, f32)| {
            if a.0 != b.0 || a.1 != b.1 {
                edges.push((a.0, a.1, b.0, b.1));
            }
        };
        for cmd in path.cmds {
            match *cmd {
                IconCmd::M(x, y) => {
                    push_seg(&mut edges, cur, sub_start);
                    cur = (x * s, y * s);
                    sub_start = cur;
                }
                IconCmd::L(x, y) => {
                    let n = (x * s, y * s);
                    push_seg(&mut edges, cur, n);
                    cur = n;
                }
                IconCmd::C(x1, y1, x2, y2, x, y) => {
                    let p0 = cur;
                    let p1 = (x1 * s, y1 * s);
                    let p2 = (x2 * s, y2 * s);
                    let p3 = (x * s, y * s);
                    let steps = 24;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let u = 1.0 - t;
                        let n = (
                            u * u * u * p0.0
                                + 3.0 * u * u * t * p1.0
                                + 3.0 * u * t * t * p2.0
                                + t * t * t * p3.0,
                            u * u * u * p0.1
                                + 3.0 * u * u * t * p1.1
                                + 3.0 * u * t * t * p2.1
                                + t * t * t * p3.1,
                        );
                        push_seg(&mut edges, cur, n);
                        cur = n;
                    }
                }
                IconCmd::Q(x1, y1, x, y) => {
                    let p0 = cur;
                    let p1 = (x1 * s, y1 * s);
                    let p2 = (x * s, y * s);
                    let steps = 16;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let u = 1.0 - t;
                        let n = (
                            u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0,
                            u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1,
                        );
                        push_seg(&mut edges, cur, n);
                        cur = n;
                    }
                }
                IconCmd::Z => {
                    push_seg(&mut edges, cur, sub_start);
                    cur = sub_start;
                }
            }
        }
        push_seg(&mut edges, cur, sub_start);
        if edges.is_empty() {
            return None;
        }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for (x0, y0, x1, y1) in &edges {
            min_x = min_x.min(*x0).min(*x1);
            min_y = min_y.min(*y0).min(*y1);
            max_x = max_x.max(*x0).max(*x1);
            max_y = max_y.max(*y0).max(*y1);
        }
        let w = (max_x - min_x).ceil() as u32 + 2;
        let h = (max_y - min_y).ceil() as u32 + 2;
        if self.cursor_x + w + 2 > ATLAS_SIZE {
            self.cursor_x = 0;
            self.cursor_y += self.row_h + 2;
            self.row_h = 0;
        }
        if self.cursor_y + h + 2 > ATLAS_SIZE {
            return None;
        }
        let ox = self.cursor_x + 1;
        let oy = self.cursor_y + 1;
        let dx = ox as f32 - min_x;
        let dy = oy as f32 - min_y;
        for py in 0..h {
            for sub_y in 0..2u32 {
                let sy = py as f32 + 0.25 + 0.5 * sub_y as f32;
                let mut crossings: Vec<(f32, i32)> = Vec::new();
                for (x0, y0, x1, y1) in &edges {
                    let (ay, by) = (y0 + dy, y1 + dy);
                    if (ay <= sy && by > sy) || (by <= sy && ay > sy) {
                        let t = (sy - ay) / (by - ay);
                        let x = x0 + dx + t * (x1 - x0);
                        let dir = if by > ay { 1 } else { -1 };
                        crossings.push((x, dir));
                    }
                }
                if crossings.is_empty() {
                    continue;
                }
                crossings
                    .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                for pxi in 0..w {
                    for sub_x in 0..2u32 {
                        let sx = pxi as f32 + 0.25 + 0.5 * sub_x as f32;
                        let mut winding = 0i32;
                        for (x, dir) in &crossings {
                            if *x > sx {
                                break;
                            }
                            winding += dir;
                        }
                        if winding != 0 {
                            let row = (oy + py) as usize * ATLAS_SIZE as usize;
                            let col = ox as usize + pxi as usize;
                            self.atlas_data[row + col] =
                                self.atlas_data[row + col].saturating_add(64);
                        }
                    }
                }
            }
        }
        self.cursor_x += w + 2;
        self.row_h = self.row_h.max(h + 2);
        self.atlas_dirty = true;
        let uv = glyph_uv(ox, oy, w, h);
        self.icon_cells.insert(key, uv);
        Some(uv)
    }

    pub fn clip_last(&mut self, clip: [f32; 4]) {
        let n = self.instances.len();
        if n >= 20 {
            let base = n - 20;
            self.instances[base + 16] = clip[0];
            self.instances[base + 17] = clip[1];
            self.instances[base + 18] = clip[2];
            self.instances[base + 19] = clip[3];
        }
    }

    pub fn text_width(&self, s: &str, px: f32, medium: bool, tracking_em: f32) -> f32 {
        let font = &self.fonts[usize::from(medium)];
        let scaled = font.as_scaled(PxScale::from(px));
        let mut width = 0.0;
        let mut prev = None;
        for c in s.chars() {
            let gid = font.glyph_id(c);
            if let Some(p) = prev {
                width += scaled.kern(p, gid);
            }
            width += scaled.h_advance(gid) + tracking_em * px;
            prev = Some(gid);
        }
        width
    }

    pub fn text(&mut self, s: &str, x: f32, y_top: f32, style: TextStyle) -> f32 {
        let px = style.px;
        let idx = usize::from(style.medium);
        let scaled = self.fonts[idx].as_scaled(PxScale::from(px));
        let a = scaled.ascent();
        let d = scaled.descent();
        let baseline = if style.line_h > 0.0 {
            y_top + (-d) + (style.line_h - (a - d)) * 0.5
        } else {
            y_top + a
        };
        let mut pen = x;
        let mut prev = None;
        for c in s.chars() {
            let gid = self.fonts[idx].glyph_id(c);
            if let Some(p) = prev {
                pen += self.fonts[idx].as_scaled(PxScale::from(px)).kern(p, gid);
            }
            if c != ' ' {
                if let Some(g) = self.rasterize(idx, c, px) {
                    self.push(Quad {
                        x: pen + g.bx,
                        y: baseline + g.by,
                        w: g.uv[2] * ATLAS_SIZE as f32,
                        h: g.uv[3] * ATLAS_SIZE as f32,
                        color: style.color,
                        uv: g.uv,
                        radius: 0.0,
                        mode: 1.0,
                        clip: [0.0; 4],
                    });
                }
            }
            pen += self.fonts[idx].as_scaled(PxScale::from(px)).h_advance(gid)
                + style.tracking_em * px;
            prev = Some(gid);
        }
        pen - x
    }

    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        clear: [f32; 4],
    ) {
        if self.atlas_dirty {
            queue.write_texture(
                TexelCopyTextureInfo {
                    texture: &self.atlas_tex,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                &self.atlas_data,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(ATLAS_SIZE),
                    rows_per_image: Some(ATLAS_SIZE),
                },
                Extent3d {
                    width: ATLAS_SIZE,
                    height: ATLAS_SIZE,
                    depth_or_array_layers: 1,
                },
            );
            self.atlas_dirty = false;
        }
        let count = self.instances.len() / 20;
        if count == 0 {
            return;
        }
        if count > self.instance_cap {
            self.instance_cap = count.next_power_of_two();
            self.instance_buf = device.create_buffer(&BufferDescriptor {
                label: None,
                size: (self.instance_cap * 80) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        let bytes: Vec<u8> = self
            .instances
            .iter()
            .flat_map(|v| v.to_ne_bytes())
            .collect();
        queue.write_buffer(&self.instance_buf, 0, &bytes);
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color {
                            r: clear[0] as f64,
                            g: clear[1] as f64,
                            b: clear[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.screen_group, &[]);
            pass.set_bind_group(1, &self.atlas_group, &[]);
            pass.set_vertex_buffer(0, self.instance_buf.slice(..));
            pass.draw(0..6, 0..count as u32);
        }
        queue.submit(Some(encoder.finish()));
        self.instances.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_paths_rasterize_with_coverage() {
        let fonts = [FontRef::try_from_slice(REGULAR).expect("regular")];
        let _ = fonts;
        for name in [
            "check",
            "close",
            "settings",
            "favorite",
            "home",
            "grid_view",
            "add",
        ] {
            let idx = icons::icon_index(name).expect("icon exists");
            let path = icons::ICONS[idx].1;
            let mut edges: Vec<(f32, f32, f32, f32)> = Vec::new();
            let mut cur = (0.0f32, 0.0f32);
            let mut start = cur;
            for cmd in path.cmds {
                match *cmd {
                    IconCmd::M(x, y) => {
                        if cur != start {
                            edges.push((cur.0, cur.1, start.0, start.1));
                        }
                        cur = (x, y);
                        start = cur;
                    }
                    IconCmd::L(x, y) => {
                        edges.push((cur.0, cur.1, x, y));
                        cur = (x, y);
                    }
                    IconCmd::C(x1, y1, x2, y2, x, y) => {
                        for i in 1..=8 {
                            let t = i as f32 / 8.0;
                            let u = 1.0 - t;
                            let n = (
                                u * u * u * cur.0
                                    + 3.0 * u * u * t * x1
                                    + 3.0 * u * t * t * x2
                                    + t * t * t * x,
                                u * u * u * cur.1
                                    + 3.0 * u * u * t * y1
                                    + 3.0 * u * t * t * y2
                                    + t * t * t * y,
                            );
                            edges.push((cur.0, cur.1, n.0, n.1));
                            cur = n;
                        }
                    }
                    IconCmd::Q(x1, y1, x, y) => {
                        for i in 1..=8 {
                            let t = i as f32 / 8.0;
                            let u = 1.0 - t;
                            let n = (
                                u * u * cur.0 + 2.0 * u * t * x1 + t * t * x,
                                u * u * cur.1 + 2.0 * u * t * y1 + t * t * y,
                            );
                            edges.push((cur.0, cur.1, n.0, n.1));
                            cur = n;
                        }
                    }
                    IconCmd::Z => {
                        edges.push((cur.0, cur.1, start.0, start.1));
                        cur = start;
                    }
                }
            }
            assert!(!edges.is_empty(), "{name} has no edges");
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            for (x0, y0, x1, y1) in &edges {
                min_x = min_x.min(*x0).min(*x1);
                max_x = max_x.max(*x0).max(*x1);
                min_y = min_y.min(*y0).min(*y1);
                max_y = max_y.max(*y0).max(*y1);
            }
            assert!(max_x - min_x > 1.0, "{name} degenerate width");
            assert!(max_y - min_y > 1.0, "{name} degenerate height");
            let n = 24usize;
            let mut covered = 0usize;
            for py in 0..n {
                let sy = py as f32 + 0.5;
                let mut xs: Vec<f32> = Vec::new();
                for (x0, y0, x1, y1) in &edges {
                    if (*y0 <= sy && *y1 > sy) || (*y1 <= sy && *y0 > sy) {
                        let t = (sy - y0) / (y1 - y0);
                        xs.push(x0 + t * (x1 - x0));
                    }
                }
                xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                for pxi in 0..n {
                    let sx = pxi as f32 + 0.5;
                    let mut inside = 0usize;
                    let mut winding = 0i32;
                    for x in &xs {
                        if *x > sx {
                            break;
                        }
                        winding += 1;
                    }
                    if winding % 2 == 1 {
                        inside += 1;
                    }
                    covered += inside;
                }
            }
            assert!(covered > 8, "{name} nearly empty raster");
        }
    }

    #[test]
    fn glyph_pixels_stay_inside_uv_window() {
        let fonts = [
            FontRef::try_from_slice(REGULAR).expect("regular"),
            FontRef::try_from_slice(MEDIUM).expect("medium"),
        ];
        for font in fonts {
            for px in [11.0f32, 13.0, 17.0, 22.0, 31.0, 46.5, 67.2] {
                let scaled = font.as_scaled(PxScale::from(px));
                for c in "Abgjy019:,.%+-()/ 10:24 Material-Geek".chars() {
                    if c == ' ' {
                        continue;
                    }
                    let mut glyph = scaled.scaled_glyph(c);
                    glyph.position = point(0.0, 0.0);
                    let Some(outlined) = scaled.outline_glyph(glyph) else {
                        continue;
                    };
                    let (gw, gh) = glyph_metrics(outlined.px_bounds());
                    if gw == 0 || gh == 0 {
                        continue;
                    }
                    let mut hits = Vec::new();
                    outlined.draw(|gx, gy, v| {
                        if v > 0.0 {
                            hits.push((gx, gy));
                        }
                    });
                    assert!(!hits.is_empty());
                    let uv = glyph_uv(1, 1, gw, gh);
                    let s = ATLAS_SIZE as f32;
                    for (gx, gy) in hits {
                        let u = (1 + gx) as f32 / s;
                        let v = (1 + gy) as f32 / s;
                        assert!(u >= uv[0] && u <= uv[0] + uv[2]);
                        assert!(v >= uv[1] && v <= uv[1] + uv[3]);
                    }
                }
            }
        }
    }
}
