use mg_components::{CatalogState, KeyInput, UiContext, UiState};
use mg_render::UiRenderer;
use std::sync::Arc;
use std::time::Instant;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

pub struct Gallery {
    pub state: UiState,
    pub catalog: CatalogState,
}

impl Default for Gallery {
    fn default() -> Self {
        Self::new()
    }
}

impl Gallery {
    pub fn new() -> Self {
        Self {
            state: UiState::default(),
            catalog: CatalogState::new(),
        }
    }

    pub fn bg(&self) -> [f32; 4] {
        self.state.pal.bg
    }

    pub fn step(&mut self, dt: f32) {
        self.state.step(dt);
    }

    pub fn scroll_by(&mut self, dy: f32) {
        self.state.scroll_by(dy);
    }

    pub fn down(&mut self, x: f32, y: f32) {
        self.state.pointer_down(x, y);
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.state.pointer_move(x, y);
    }

    pub fn up(&mut self, x: f32, y: f32) {
        self.state.pointer_up(x, y);
    }

    pub fn key_input(&mut self, key: KeyInput) {
        self.state.push_key(key);
    }

    pub fn draw(&mut self, ui: &mut UiRenderer, width: f32, height: f32) {
        let mut ctx = UiContext::begin(ui, &mut self.state, width, height);
        ctx.draw_catalog(&mut self.catalog);
    }
}

pub struct GalleryApp {
    gallery: Gallery,
    ui: Option<UiInner>,
    last_frame: Option<Instant>,
}

struct UiInner {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    renderer: UiRenderer,
}

impl Default for GalleryApp {
    fn default() -> Self {
        Self::new()
    }
}

impl GalleryApp {
    pub fn new() -> Self {
        Self {
            gallery: Gallery::new(),
            ui: None,
            last_frame: None,
        }
    }

    pub fn resume(&mut self, window: Arc<Window>) -> bool {
        use log::{error, info};
        use wgpu::{
            CompositeAlphaMode, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
            InstanceDescriptor, MemoryHints, PowerPreference, PresentMode, RequestAdapterOptions,
            TextureFormat, TextureUsages, Trace,
        };
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let surface = match instance.create_surface(window.clone()) {
            Ok(surface) => surface,
            Err(err) => {
                error!("surface creation failed: {err:?}");
                return false;
            }
        };
        let adapter = match pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
            apply_limit_buckets: false,
        })) {
            Ok(adapter) => adapter,
            Err(err) => {
                error!("adapter request failed: {err:?}");
                return false;
            }
        };
        let (device, queue) = match pollster::block_on(adapter.request_device(&DeviceDescriptor {
            label: None,
            required_features: Features::empty(),
            required_limits: adapter.limits(),
            experimental_features: ExperimentalFeatures::disabled(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
        })) {
            Ok(pair) => pair,
            Err(err) => {
                error!("device request failed: {err:?}");
                return false;
            }
        };
        let caps = surface.get_capabilities(&adapter);
        let size = window.inner_size();
        let format = caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(TextureFormat::Bgra8UnormSrgb);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(CompositeAlphaMode::Auto),
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let mut renderer = UiRenderer::new(&device, &queue, config.format);
        renderer.set_screen(&device, &queue, config.width as f32, config.height as f32);
        info!("gallery renderer ready {}x{}", config.width, config.height);
        self.last_frame = None;
        self.ui = Some(UiInner {
            window,
            surface,
            device,
            queue,
            config,
            renderer,
        });
        true
    }

    pub fn suspend(&mut self) {
        self.ui = None;
        self.last_frame = None;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(ui) = self.ui.as_mut() {
            ui.config.width = width.max(1);
            ui.config.height = height.max(1);
            ui.surface.configure(&ui.device, &ui.config);
            ui.renderer.set_screen(
                &ui.device,
                &ui.queue,
                ui.config.width as f32,
                ui.config.height as f32,
            );
        }
    }

    pub fn frame(&mut self) {
        use wgpu::CurrentSurfaceTexture;
        let Some(ui) = self.ui.as_mut() else {
            return;
        };
        let now = Instant::now();
        let dt = self
            .last_frame
            .map_or(1.0 / 60.0, |prev| (now - prev).as_secs_f32());
        self.last_frame = Some(now);
        self.gallery.step(dt);
        let size = ui.window.inner_size();
        if size.width != ui.config.width || size.height != ui.config.height {
            ui.config.width = size.width.max(1);
            ui.config.height = size.height.max(1);
            ui.surface.configure(&ui.device, &ui.config);
            ui.renderer.set_screen(
                &ui.device,
                &ui.queue,
                ui.config.width as f32,
                ui.config.height as f32,
            );
        }
        let frame = match ui.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) | CurrentSurfaceTexture::Suboptimal(frame) => {
                frame
            }
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                ui.config.width = size.width.max(1);
                ui.config.height = size.height.max(1);
                ui.surface.configure(&ui.device, &ui.config);
                ui.renderer.set_screen(
                    &ui.device,
                    &ui.queue,
                    ui.config.width as f32,
                    ui.config.height as f32,
                );
                ui.window.request_redraw();
                return;
            }
            _ => {
                ui.window.request_redraw();
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.gallery.draw(
            &mut ui.renderer,
            ui.config.width as f32,
            ui.config.height as f32,
        );
        ui.renderer
            .flush(&ui.device, &ui.queue, &view, self.gallery.bg());
        ui.queue.present(frame);
        ui.window.request_redraw();
    }

    pub fn scroll(&mut self, dy: f32) {
        self.gallery.scroll_by(dy);
    }

    pub fn pointer_down(&mut self, x: f32, y: f32) {
        self.gallery.down(x, y);
    }

    pub fn pointer_move(&mut self, x: f32, y: f32) {
        self.gallery.move_to(x, y);
    }

    pub fn pointer_up(&mut self, x: f32, y: f32) {
        self.gallery.up(x, y);
    }

    pub fn key_input(&mut self, key: KeyInput) {
        self.gallery.key_input(key);
    }
}
