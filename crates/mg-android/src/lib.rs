mod gallery;

use gallery::GalleryApp;
use log::{error, info};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, Touch, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;

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

struct Handler {
    app: GalleryApp,
    mouse: (f32, f32),
    mouse_down: bool,
}

impl Handler {
    fn new() -> Self {
        Self {
            app: GalleryApp::new(),
            mouse: (0.0, 0.0),
            mouse_down: false,
        }
    }
}

impl ApplicationHandler for Handler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        let window = match event_loop.create_window(Window::default_attributes()) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                error!("window creation failed: {err:?}");
                return;
            }
        };
        let size = window.inner_size();
        info!("resumed with window {}x{}", size.width, size.height);
        if !self.app.resume(window) {
            error!("gallery init failed, will retry on next resume");
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        info!("suspended, dropping surface");
        self.app.suspend();
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
                self.app.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                self.app.frame();
            }
            WindowEvent::Touch(Touch { phase, location, .. }) => {
                let x = location.x as f32;
                let y = location.y as f32;
                match phase {
                    TouchPhase::Started => self.app.pointer_down(x, y),
                    TouchPhase::Moved => self.app.pointer_move(x, y),
                    TouchPhase::Ended | TouchPhase::Cancelled => self.app.pointer_up(x, y),
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse = (position.x as f32, position.y as f32);
                if self.mouse_down {
                    self.app.pointer_move(self.mouse.0, self.mouse.1);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            self.mouse_down = true;
                            self.app.pointer_down(self.mouse.0, self.mouse.1);
                        }
                        ElementState::Released => {
                            self.mouse_down = false;
                            self.app.pointer_up(self.mouse.0, self.mouse.1);
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -y * 60.0,
                    MouseScrollDelta::PixelDelta(p) => -(p.y as f32),
                };
                self.app.scroll(dy);
            }
            _ => {}
        }
    }
}

#[cfg(not(target_os = "android"))]
pub fn run_desktop() {
    init_logging();
    let event_loop = EventLoop::builder().build().expect("event loop");
    let mut handler = Handler::new();
    let _ = event_loop.run_app(&mut handler);
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    use mg_theme::GeekTheme;
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
    let mut handler = Handler::new();
    match event_loop.run_app(&mut handler) {
        Ok(()) => info!("event loop returned, ending android thread"),
        Err(err) => error!("event loop exited with error: {err:?}"),
    }
}
