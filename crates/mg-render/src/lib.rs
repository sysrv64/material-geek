use ab_glyph::{point, Font, FontRef, PxScale, ScaleFont};
use std::collections::HashMap;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BlendState,
    Buffer, BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites, Extent3d,
    FilterMode, FragmentState, MultisampleState, Origin3d, RenderPipeline,
    RenderPipelineDescriptor, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode,
};

const REGULAR: &[u8] = include_bytes!("../assets/Roboto-Regular.ttf");
const MEDIUM: &[u8] = include_bytes!("../assets/Roboto-Medium.ttf");
const ATLAS_SIZE: u32 = 1024;

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
};
struct VOut {
    @builtin(position) pos: vec4f,
    @location(0) corner: vec2f,
    @location(1) color: vec4f,
    @location(2) uv: vec2f,
    @location(3) uv_size: vec2f,
    @location(4) misc: vec4f,
    @location(5) half_size: vec2f,
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
    return vec4f(col.rgb, col.a * mask);
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
}

#[derive(Clone, Copy)]
pub struct TextStyle {
    pub px: f32,
    pub medium: bool,
    pub color: [f32; 4],
    pub tracking_em: f32,
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
        ];
        let instance_layout = VertexBufferLayout {
            array_stride: 64,
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
            size: 64 * 256,
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
        });
    }

    fn push(&mut self, q: Quad) {
        self.instances.extend_from_slice(&[
            q.x, q.y, q.w, q.h, q.color[0], q.color[1], q.color[2], q.color[3], q.uv[0], q.uv[1],
            q.uv[2], q.uv[3], q.radius, q.mode, 0.0, 0.0,
        ]);
    }

    fn rasterize(&mut self, font_idx: usize, c: char, px: f32) -> Option<GlyphEntry> {
        let font = &self.fonts[font_idx];
        let gid = font.glyph_id(c);
        let key = GlyphKey {
            font: font_idx as u8,
            id: gid.0,
            px: px as u32,
        };
        if let Some(entry) = self.glyphs.get(&key) {
            return Some(*entry);
        }
        let scaled = font.as_scaled(PxScale::from(px));
        let mut glyph = scaled.scaled_glyph(c);
        glyph.position = point(0.0, 0.0);
        let outlined = scaled.outline_glyph(glyph)?;
        let bounds = outlined.px_bounds();
        let w = (bounds.width().ceil() as u32) + 2;
        let h = (bounds.height().ceil() as u32) + 2;
        if self.cursor_x + w > ATLAS_SIZE {
            self.cursor_x = 0;
            self.cursor_y += self.row_h + 2;
            self.row_h = 0;
        }
        if self.cursor_y + h > ATLAS_SIZE {
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
        self.cursor_x += w + 2;
        self.row_h = self.row_h.max(h + 2);
        self.atlas_dirty = true;
        let entry = GlyphEntry {
            uv: [
                (ox as f32 + bounds.min.x) / ATLAS_SIZE as f32,
                (oy as f32 + bounds.min.y) / ATLAS_SIZE as f32,
                bounds.width() / ATLAS_SIZE as f32,
                bounds.height() / ATLAS_SIZE as f32,
            ],
            bx: bounds.min.x,
            by: bounds.min.y,
        };
        self.glyphs.insert(key, entry);
        Some(entry)
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
        let ascent = self.fonts[idx].as_scaled(PxScale::from(px)).ascent();
        let baseline = y_top + ascent;
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
        let count = self.instances.len() / 16;
        if count == 0 {
            return;
        }
        if count > self.instance_cap {
            self.instance_cap = count.next_power_of_two();
            self.instance_buf = device.create_buffer(&BufferDescriptor {
                label: None,
                size: (self.instance_cap * 64) as u64,
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
