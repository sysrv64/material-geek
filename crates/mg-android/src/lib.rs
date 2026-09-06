use log::{error, info};
use mg_motion::{DynamicSpring, MotionScheme, Tempo, Track};
use mg_theme::GeekTheme;
use std::sync::Arc;
use wgpu::{
    Adapter, BindGroupDescriptor, BindGroupEntry, BindingResource, BlendState, BufferBinding,
    BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor,
    CompositeAlphaMode, CurrentSurfaceTexture, Device, DeviceDescriptor, ExperimentalFeatures,
    Features, FragmentState, Instance, InstanceDescriptor, LoadOp, MemoryHints, MultisampleState,
    Operations, PowerPreference, PresentMode, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, StoreOp, Surface,
    SurfaceColorSpace, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor,
    Trace, VertexState,
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;

const SHADER: &str = r#"
struct Scene {
    bg: vec4f,
    fg: vec4f,
    circle: vec4f,
};
@group(0) @binding(0) var<uniform> scene: Scene;
struct VsOut {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f,
};
@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
    var pos = vec2f(-1.0, -1.0);
    if (i == 1u) {
        pos = vec2f(3.0, -1.0);
    } else if (i == 2u) {
        pos = vec2f(-1.0, 3.0);
    }
    var out: VsOut;
    out.pos = vec4f(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + vec2f(0.5, 0.5);
    return out;
}
@fragment
fn fs(in: VsOut) -> @location(0) vec4f {
    let aspect = scene.circle.w;
    let p = vec2f(in.uv.x * aspect, in.uv.y);
    let c = vec2f(scene.circle.x * aspect, scene.circle.y);
    let d = distance(p, c);
    let edge = 1.0 - smoothstep(scene.circle.z - 0.008, scene.circle.z, d);
    return mix(scene.bg, scene.fg, edge);
}
"#;

fn srgb_to_linear(byte: u8) -> f32 {
    let f = byte as f32 / 255.0;
    if f <= 0.04045 {
        f / 12.92
    } else {
        ((f + 0.055) / 1.055).powf(2.4)
    }
}

fn argb_to_linear(argb: u32) -> [f32; 4] {
    [
        srgb_to_linear(((argb >> 16) & 0xFF) as u8),
        srgb_to_linear(((argb >> 8) & 0xFF) as u8),
        srgb_to_linear((argb & 0xFF) as u8),
        1.0,
    ]
}

fn uniform_bytes(bg: [f32; 4], fg: [f32; 4], circle: [f32; 4]) -> [u8; 48] {
    let mut out = [0u8; 48];
    for (i, v) in bg.iter().chain(fg.iter()).chain(circle.iter()).enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_ne_bytes());
    }
    out
}

fn init_logging() {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("material-geek")
            .with_max_level(log::LevelFilter::Debug),
    );
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("panic: {info}");
        previous(info);
    }));
}

fn request_adapter(instance: &Instance, surface: &Surface<'static>) -> Option<Adapter> {
    let options = RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(surface),
        apply_limit_buckets: false,
    };
    if let Ok(adapter) = pollster::block_on(instance.request_adapter(&options)) {
        return Some(adapter);
    }
    error!("high performance adapter unavailable, trying fallback");
    pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::LowPower,
        force_fallback_adapter: true,
        compatible_surface: Some(surface),
        apply_limit_buckets: false,
    }))
    .map_err(|err| error!("adapter request failed: {err:?}"))
    .ok()
}

struct RenderState {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bg: [f32; 4],
    fg: [f32; 4],
    spring: DynamicSpring,
}

impl RenderState {
    fn new(window: Arc<Window>, theme: &GeekTheme) -> Option<Self> {
        let started = std::time::Instant::now();
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let surface = match instance.create_surface(window.clone()) {
            Ok(surface) => surface,
            Err(err) => {
                error!("surface creation failed: {err:?}");
                return None;
            }
        };
        info!("stage surface ok in {}ms", started.elapsed().as_millis());
        let adapter = request_adapter(&instance, &surface)?;
        info!("stage adapter ok in {}ms", started.elapsed().as_millis());
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
                return None;
            }
        };
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .first()
            .copied()
            .unwrap_or(TextureFormat::Bgra8Unorm);
        let alpha = caps
            .alpha_modes
            .first()
            .copied()
            .unwrap_or(CompositeAlphaMode::Auto);
        let size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: SurfaceColorSpace::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: alpha,
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        info!("stage device ok in {}ms", started.elapsed().as_millis());
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(SHADER.into()),
        });
        info!("stage shader ok in {}ms", started.elapsed().as_millis());
        let uniform_buf = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 48,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let bind_layout = pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &uniform_buf,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        let bg = argb_to_linear(theme.scheme.surface.0);
        let fg = argb_to_linear(theme.scheme.primary.0);
        let spec = MotionScheme::expressive().spec(Tempo::Default, Track::Spatial);
        info!(
            "renderer ready {}x{} format={:?} in {}ms",
            config.width,
            config.height,
            config.format,
            started.elapsed().as_millis()
        );
        Some(Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            uniform_buf,
            bind_group,
            bg,
            fg,
            spring: DynamicSpring::new(0.0, 1.0, spec),
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    fn frame(&mut self) {
        self.spring.step(1.0 / 60.0);
        if self.spring.settled() {
            let next = if self.spring.target >= 1.0 { 0.0 } else { 1.0 };
            self.spring.retarget(next);
        }
        let aspect = self.config.width as f32 / self.config.height as f32;
        let margin = 0.2;
        let circle = [
            margin + self.spring.x * (1.0 - 2.0 * margin),
            0.5,
            0.085,
            aspect,
        ];
        let bytes = uniform_bytes(self.bg, self.fg, circle);
        self.queue.write_buffer(&self.uniform_buf, 0, &bytes);
        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) | CurrentSurfaceTexture::Suboptimal(frame) => {
                frame
            }
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                let size = self.window.inner_size();
                self.resize(size.width, size.height);
                self.window.request_redraw();
                return;
            }
            _ => {
                self.window.request_redraw();
                return;
            }
        };
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: self.bg[0] as f64,
                            g: self.bg[1] as f64,
                            b: self.bg[2] as f64,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        self.window.request_redraw();
    }
}

struct GeekApp {
    state: Option<RenderState>,
    theme: GeekTheme,
}

impl GeekApp {
    fn new() -> Self {
        Self {
            state: None,
            theme: GeekTheme::dark_expressive(),
        }
    }
}

impl ApplicationHandler for GeekApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        if self.state.is_none() {
            let window = match event_loop.create_window(Window::default_attributes()) {
                Ok(window) => Arc::new(window),
                Err(err) => {
                    error!("window creation failed: {err:?}");
                    return;
                }
            };
            info!(
                "resumed with window {}x{}",
                window.inner_size().width,
                window.inner_size().height
            );
            match RenderState::new(window, &self.theme) {
                Some(state) => self.state = Some(state),
                None => error!("renderer init failed, will retry on next resume"),
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        info!("suspended, dropping surface");
        self.state = None;
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                #[cfg(not(target_os = "android"))]
                _event_loop.exit();
                #[cfg(target_os = "android")]
                info!("close requested, keeping loop alive for activity reuse");
            }
            WindowEvent::Resized(size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = self.state.as_mut() {
                    state.frame();
                }
            }
            _ => {}
        }
    }
}

#[cfg(not(target_os = "android"))]
pub fn run_desktop() {
    init_logging();
    let event_loop = EventLoop::builder().build().expect("event loop");
    let mut handler = GeekApp::new();
    let _ = event_loop.run_app(&mut handler);
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    init_logging();
    info!("material-geek android entry");
    if let Some(dir) = app.internal_data_path() {
        let theme = GeekTheme::dark_expressive();
        let payload = format!(
            "dark={} motion={:?} components=41\n",
            theme.scheme.is_dark,
            theme.scheme_kind()
        );
        let _ = std::fs::write(dir.join("material-geek-status.txt"), payload);
    }
    let mut builder = EventLoop::builder();
    builder.with_android_app(app);
    let event_loop = match builder.build() {
        Ok(event_loop) => event_loop,
        Err(err) => {
            error!("event loop creation failed: {err:?}");
            if matches!(err, winit::error::EventLoopError::RecreationAttempt) {
                error!("event loop already exists in this process, restarting process");
                std::process::exit(0);
            }
            return;
        }
    };
    let mut handler = GeekApp::new();
    match event_loop.run_app(&mut handler) {
        Ok(()) => info!("event loop returned, ending android thread"),
        Err(err) => error!("event loop exited with error: {err:?}"),
    }
}
