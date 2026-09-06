mod icon_names;
mod shape;
mod text;

pub use icon_names::icon_codepoint;
pub use shape::ShapeInstance;
pub use text::TextEngine;

use shape::{ShapePipeline, DEPTH_FORMAT};
use text::TextEngine as Engine;
use wgpu::{
    Device, Extent3d, LoadOp, Operations, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView,
};

pub use glyphon;

const REGULAR: &[u8] = include_bytes!("../assets/Roboto-Regular.ttf");
const MEDIUM: &[u8] = include_bytes!("../assets/Roboto-Medium.ttf");
const SYMBOLS_TTF: &[u8] = include_bytes!("../assets/MaterialSymbols-Regular.ttf");

pub const ROBOTO_FAMILY: &str = "Roboto";
pub const SYMBOLS_FAMILY: &str = "Material Symbols Outlined";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub px: f32,
    pub medium: bool,
    pub color: [f32; 4],
    pub tracking_em: f32,
    pub line_h: f32,
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

fn srgb_channel_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn clear_to_linear(c: [f32; 4]) -> wgpu::Color {
    wgpu::Color {
        r: srgb_channel_to_linear(c[0]) as f64,
        g: srgb_channel_to_linear(c[1]) as f64,
        b: srgb_channel_to_linear(c[2]) as f64,
        a: c[3] as f64,
    }
}

const LAYER_EPS: f32 = 1e-5;

pub(crate) fn layer_depth(n: u32) -> f32 {
    (1.0 - (n as f32 + 1.0) * LAYER_EPS).max(0.0)
}

pub struct UiRenderer {
    shape: ShapePipeline,
    engine: Engine,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,
    depth_view: TextureView,
    depth_size: (u32, u32),
    layer: u32,
}

impl UiRenderer {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let shape = ShapePipeline::new(device, format);
        let engine = TextEngine::new();
        let cache = glyphon::Cache::new(device);
        let viewport = glyphon::Viewport::new(device, &cache);
        let mut atlas = glyphon::TextAtlas::new(device, queue, &cache, format);
        let text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
        );
        let depth_tex = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let mut renderer = Self {
            shape,
            engine,
            viewport,
            atlas,
            text_renderer,
            depth_view: depth_tex.create_view(&wgpu::TextureViewDescriptor::default()),
            depth_size: (0, 0),
            layer: 0,
        };
        renderer.set_screen(device, queue, 1.0, 1.0);
        renderer
    }

    pub fn set_screen(&mut self, device: &Device, queue: &Queue, w: f32, h: f32) {
        self.shape.set_screen(queue, w, h);
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: w.max(1.0) as u32,
                height: h.max(1.0) as u32,
            },
        );
        let size = (w.max(1.0) as u32, h.max(1.0) as u32);
        if self.depth_size != size {
            self.depth_size = size;
            let tex = device.create_texture(&TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            self.depth_view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    fn next_depth(&mut self) -> f32 {
        self.layer += 1;
        layer_depth(self.layer)
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4], radii: [f32; 4]) {
        let mut inst = ShapeInstance::empty();
        inst.rect = [x, y, w, h];
        inst.fill_col = color;
        inst.radii = radii;
        inst._pad = [self.next_depth(), 0.0];
        self.shape.instances.push(inst);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn stroke(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        radii: [f32; 4],
        stroke_width: f32,
    ) {
        let mut inst = ShapeInstance::empty();
        inst.rect = [x, y, w, h];
        inst.stroke_col = color;
        inst.radii = radii;
        inst.stroke_clip = [0.0, 0.0, stroke_width, 0.0];
        inst._pad = [self.next_depth(), 0.0];
        self.shape.instances.push(inst);
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, color: [f32; 4]) {
        self.rect(cx - r, cy - r, r * 2.0, r * 2.0, color, [r; 4]);
    }

    pub fn clip_last(&mut self, clip: [f32; 4]) {
        self.shape.clip_last(clip);
    }

    pub fn text(&mut self, s: &str, x: f32, y_top: f32, style: TextStyle) -> f32 {
        let layer = self.layer + 1;
        let width = self.engine.text(s, x, y_top, style, layer);
        self.layer = layer;
        width
    }

    pub fn text_width(&mut self, s: &str, px: f32, medium: bool, tracking_em: f32) -> f32 {
        self.engine.text_width(s, px, medium, tracking_em)
    }

    pub fn icon(&mut self, name: &str, x: f32, y: f32, px: f32, color: [f32; 4]) {
        self.layer += 1;
        self.engine.icon(name, x, y, px, color, self.layer);
    }

    pub fn flush(&mut self, device: &Device, queue: &Queue, view: &TextureView, clear: [f32; 4]) {
        let count = self.shape.upload(device, queue);
        self.engine.prepare(
            &mut self.text_renderer,
            &mut self.atlas,
            &self.viewport,
            device,
            queue,
        );
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(clear_to_linear(clear)),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if count > 0 {
                self.shape.draw(&mut pass);
            }
            if let Err(err) = self
                .text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
            {
                log::warn!("glyphon render failed: {err:?}");
            }
        }
        queue.submit(Some(encoder.finish()));
        self.shape.clear();
        self.engine.clear_entries();
        self.atlas.trim();
        self.layer = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_names_resolve() {
        assert!(icon_codepoint("settings").is_some());
        assert!(icon_codepoint("home").is_some());
        assert!(icon_codepoint("check").is_some());
        assert!(icon_codepoint("favorite").is_some());
        assert!(icon_codepoint("more_vert").is_some());
        assert_eq!(icon_codepoint("definitely_not_an_icon"), None);
        assert_eq!(icon_codepoint(""), None);
    }

    #[test]
    fn srgb_to_linear_helpers_match() {
        assert!((srgb_channel_to_linear(0.0)).abs() < f32::EPSILON);
        assert!((srgb_channel_to_linear(1.0) - 1.0).abs() < 1e-6);
        let lin = srgb_channel_to_linear(0.5);
        assert!((0.21..0.22).contains(&lin));
    }

    #[test]
    fn text_style_consts_keep_m3_scale() {
        assert_eq!(M3_DISPLAY_LARGE.px, 57.0);
        assert_eq!(M3_BODY_LARGE.line_h, 24.0);
    }
}
