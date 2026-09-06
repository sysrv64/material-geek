use crate::icon_names::icon_codepoint;
use crate::{TextStyle, MEDIUM, REGULAR, ROBOTO_FAMILY, SYMBOLS_FAMILY, SYMBOLS_TTF};
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, Weight};
use glyphon::{Color, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Device, Queue};

const TEXT_CACHE_CAP: usize = 512;
const ICON_CACHE_CAP: usize = 256;

#[derive(Clone, PartialEq, Eq, Hash)]
struct TextKey {
    content: Arc<str>,
    px_bits: u32,
    line_h_bits: u32,
    tracking_bits: u32,
    medium: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct IconKey {
    name: Arc<str>,
    px_bits: u32,
}

#[derive(Clone)]
enum BufSource {
    Text(TextKey),
    Icon(IconKey),
}

struct TextEntry {
    src: BufSource,
    meta: usize,
    layer: u32,
    x: f32,
    y: f32,
    color: [f32; 4],
}

pub struct TextEngine {
    pub font_system: FontSystem,
    swash: SwashCache,
    buffers: HashMap<TextKey, Buffer>,
    icon_buffers: HashMap<IconKey, Buffer>,
    entries: Vec<TextEntry>,
    next_meta: usize,
}

fn to_u8(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEngine {
    pub fn new() -> Self {
        let mut font_system = FontSystem::new();
        font_system.db_mut().load_font_data(REGULAR.to_vec());
        font_system.db_mut().load_font_data(MEDIUM.to_vec());
        font_system.db_mut().load_font_data(SYMBOLS_TTF.to_vec());
        Self {
            font_system,
            swash: SwashCache::new(),
            buffers: HashMap::new(),
            icon_buffers: HashMap::new(),
            entries: Vec::new(),
            next_meta: 1,
        }
    }

    fn text_buffer(
        &mut self,
        s: &str,
        px: f32,
        line_h: f32,
        medium: bool,
        tracking_em: f32,
    ) -> (TextKey, usize) {
        let key = TextKey {
            content: Arc::from(s),
            px_bits: px.to_bits(),
            line_h_bits: line_h.to_bits(),
            tracking_bits: tracking_em.to_bits(),
            medium,
        };
        if !self.buffers.contains_key(&key) {
            if self.buffers.len() >= TEXT_CACHE_CAP {
                self.buffers.clear();
            }
            let meta = self.next_meta;
            self.next_meta += 1;
            let lh = if line_h > 0.0 { line_h } else { px * 1.3 };
            let attrs = Attrs::new()
                .family(Family::Name(ROBOTO_FAMILY))
                .weight(if medium {
                    Weight::MEDIUM
                } else {
                    Weight::NORMAL
                })
                .letter_spacing(tracking_em * px)
                .metadata(meta);
            let mut buf = Buffer::new(&mut self.font_system, Metrics::new(px, lh));
            buf.set_text(s, &attrs, Shaping::Advanced, None);
            buf.shape_until_scroll(&mut self.font_system, true);
            self.buffers.insert(key.clone(), buf);
        }
        let buf = self
            .buffers
            .get(&key)
            .expect("shaped buffer present after text_buffer");
        let meta = buf
            .layout_runs()
            .next()
            .and_then(|r| r.glyphs.first())
            .map_or(0, |g| g.metadata);
        (key, meta)
    }

    pub fn text(&mut self, s: &str, x: f32, y_top: f32, style: TextStyle, layer: u32) -> f32 {
        let (key, meta) =
            self.text_buffer(s, style.px, style.line_h, style.medium, style.tracking_em);
        let width = {
            let buf = self
                .buffers
                .get(&key)
                .expect("shaped buffer present after text_buffer");
            buf.layout_runs().next().map_or(0.0, |r| r.line_w)
        };
        self.entries.push(TextEntry {
            src: BufSource::Text(key),
            meta,
            layer,
            x,
            y: y_top,
            color: style.color,
        });
        width
    }

    pub fn text_width(&mut self, s: &str, px: f32, medium: bool, tracking_em: f32) -> f32 {
        let (key, _) = self.text_buffer(s, px, px, medium, tracking_em);
        let buf = self
            .buffers
            .get(&key)
            .expect("shaped buffer present after text_buffer");
        buf.layout_runs().next().map_or(0.0, |r| r.line_w)
    }

    pub fn icon(&mut self, name: &str, x: f32, y: f32, px: f32, color: [f32; 4], layer: u32) {
        let Some(ch) = icon_codepoint(name) else {
            log::warn!("unknown icon name: {name}");
            return;
        };
        let key = IconKey {
            name: Arc::from(name),
            px_bits: px.to_bits(),
        };
        if !self.icon_buffers.contains_key(&key) {
            if self.icon_buffers.len() >= ICON_CACHE_CAP {
                self.icon_buffers.clear();
            }
            let meta = self.next_meta;
            self.next_meta += 1;
            let attrs = Attrs::new()
                .family(Family::Name(SYMBOLS_FAMILY))
                .metadata(meta);
            let mut buf = Buffer::new(&mut self.font_system, Metrics::new(px, px));
            buf.set_text(&ch.to_string(), &attrs, Shaping::Advanced, None);
            buf.shape_until_scroll(&mut self.font_system, true);
            self.icon_buffers.insert(key.clone(), buf);
        }
        let buf = self
            .icon_buffers
            .get(&key)
            .expect("icon buffer present after insert");
        let meta = buf
            .layout_runs()
            .next()
            .and_then(|r| r.glyphs.first())
            .map_or(0, |g| g.metadata);
        self.entries.push(TextEntry {
            src: BufSource::Icon(key),
            meta,
            layer,
            x,
            y,
            color,
        });
    }

    pub fn prepare(
        &mut self,
        renderer: &mut TextRenderer,
        atlas: &mut TextAtlas,
        viewport: &Viewport,
        device: &Device,
        queue: &Queue,
    ) {
        let mut depth_map: HashMap<usize, f32> = HashMap::with_capacity(self.entries.len());
        for e in &self.entries {
            depth_map.insert(e.meta, crate::layer_depth(e.layer));
        }
        let TextEngine {
            font_system,
            swash,
            buffers,
            icon_buffers,
            entries,
            ..
        } = self;
        let areas: Vec<TextArea<'_>> = entries
            .iter()
            .map(|e| {
                let buffer = match &e.src {
                    BufSource::Text(k) => &buffers[k],
                    BufSource::Icon(k) => &icon_buffers[k],
                };
                let [r, g, b, a] = e.color;
                TextArea {
                    buffer,
                    left: e.x,
                    top: e.y,
                    scale: 1.0,
                    bounds: TextBounds::default(),
                    default_color: Color::rgba(to_u8(r), to_u8(g), to_u8(b), to_u8(a)),
                    custom_glyphs: &[],
                }
            })
            .collect();
        if let Err(err) = renderer.prepare_with_depth(
            device,
            queue,
            font_system,
            atlas,
            viewport,
            areas,
            swash,
            |meta| {
                depth_map
                    .get(&meta)
                    .copied()
                    .unwrap_or(crate::layer_depth(0))
            },
        ) {
            log::warn!("glyphon prepare failed: {err:?}");
        }
    }

    pub fn clear_entries(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> TextEngine {
        TextEngine::new()
    }

    #[test]
    fn text_width_is_positive_and_cached() {
        let mut e = engine();
        let w1 = e.text_width("Material", 14.0, false, 0.0);
        assert!(w1 > 10.0);
        let w2 = e.text_width("Material", 14.0, false, 0.0);
        assert_eq!(w1, w2);
        assert_eq!(e.buffers.len(), 1);
    }

    #[test]
    fn tracking_changes_width() {
        let mut e = engine();
        let plain = e.text_width("gg", 16.0, false, 0.0);
        let tracked = e.text_width("gg", 16.0, false, 0.5);
        assert!(tracked > plain);
    }

    #[test]
    fn medium_weight_differers_or_at_least_shapes() {
        let mut e = engine();
        let a = e.text_width("Hello", 16.0, false, 0.0);
        let b = e.text_width("Hello", 16.0, true, 0.0);
        assert!(a > 0.0 && b > 0.0);
    }

    #[test]
    fn text_entry_queued_for_each_call() {
        let mut e = engine();
        let style = TextStyle {
            px: 14.0,
            medium: false,
            color: [1.0, 1.0, 1.0, 1.0],
            tracking_em: 0.0,
            line_h: 20.0,
        };
        e.text("Row one", 0.0, 0.0, style, 1);
        e.text("Row two", 0.0, 24.0, style, 2);
        assert_eq!(e.entries.len(), 2);
        e.clear_entries();
        assert!(e.entries.is_empty());
    }

    #[test]
    fn cache_clears_when_over_cap() {
        let mut e = engine();
        for i in 0..(TEXT_CACHE_CAP + 8) {
            let _ = e.text_width(&format!("s{i}"), 12.0, false, 0.0);
        }
        assert!(e.buffers.len() < TEXT_CACHE_CAP);
    }
}
