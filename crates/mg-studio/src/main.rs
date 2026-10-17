//! Material Geek Studio — separate PC application for the Material 3
//! Expressive catalog. Runs the same GPU component stack as the phone app,
//! but in the adaptive list-detail layout at desktop pixel density, with
//! hardware keyboard input and no phone chrome (status-bar inset, soft
//! keyboard).

use log::info;
use mg_android::{GalleryApp, KeyInput};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{
    ElementState, Ime, KeyEvent, MouseButton, MouseScrollDelta, Touch, TouchPhase, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

/// Device pixel density of the studio window in px/dp; desktop monitors are
/// roughly 96–160 dpi so a modest fixed scale reads far better than the
/// width-derived phone scaling (see `UiState::density`).
const STUDIO_DENSITY: f32 = 1.3;

/// Initial window size in logical (winit CSS) px: at density 1.3 this maps
/// to ~923 logical dp wide, past the 840dp adaptive list-detail breakpoint.
const STUDIO_WINDOW_W_PX: f64 = 1200.0;
const STUDIO_WINDOW_H_PX: f64 = 800.0;

struct StudioApp {
    app: GalleryApp,
    mouse: (f32, f32),
    mouse_down: bool,
}

impl StudioApp {
    fn new() -> Self {
        let mut app = GalleryApp::new();
        let state = &mut app.gallery_mut().state;
        state.density = STUDIO_DENSITY;
        state.top_inset_dp = 0.0;
        state.soft_keyboard = false;
        Self {
            app,
            mouse: (0.0, 0.0),
            mouse_down: false,
        }
    }
}

impl ApplicationHandler for StudioApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_title("Material Geek Studio — M3 Expressive")
                .with_inner_size(LogicalSize::new(STUDIO_WINDOW_W_PX, STUDIO_WINDOW_H_PX))
                .with_min_inner_size(LogicalSize::new(800.0, 600.0)),
        ) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                info!("window creation failed: {err:?}");
                event_loop.exit();
                return;
            }
        };
        window.set_ime_allowed(true);
        if !self.app.resume(window) {
            info!("gallery init failed, will retry on next resume");
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.app.suspend();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.app.resize(size.width, size.height),
            WindowEvent::RedrawRequested => self.app.frame(),
            WindowEvent::Ime(Ime::Commit(text)) => self.app.key_input(KeyInput::Text(text)),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match logical_key {
                Key::Named(NamedKey::Backspace) => self.app.key_input(KeyInput::Backspace),
                Key::Named(NamedKey::Delete) => self.app.key_input(KeyInput::Delete),
                Key::Named(NamedKey::ArrowLeft) => self.app.key_input(KeyInput::Left),
                Key::Named(NamedKey::ArrowRight) => self.app.key_input(KeyInput::Right),
                Key::Named(NamedKey::Home) => self.app.key_input(KeyInput::Home),
                Key::Named(NamedKey::End) => self.app.key_input(KeyInput::End),
                Key::Named(NamedKey::Enter) => self.app.key_input(KeyInput::Enter),
                Key::Named(NamedKey::Escape) => self.app.key_input(KeyInput::Escape),
                Key::Named(NamedKey::Space) => self.app.key_input(KeyInput::Char(' ')),
                Key::Character(s) => self.app.key_input(KeyInput::Text(s.to_string())),
                _ => {}
            },
            WindowEvent::Touch(Touch {
                phase, location, ..
            }) => {
                let (x, y) = (location.x as f32, location.y as f32);
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

fn run_studio() {
    log::set_max_level(log::LevelFilter::Info);
    let event_loop = EventLoop::builder().build().expect("event loop");
    let mut app = StudioApp::new();
    let _ = event_loop.run_app(&mut app);
}

fn main() {
    run_studio();
}
