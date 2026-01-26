//! Device Emulation Workbench
//!
//! A lightweight tool for iterative device modeling with Claude integration.
//! Exposes an HTTP API for shader control, screenshots, and visual comparison.

mod api;
mod diff;
mod renderer;
mod state;

use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use crate::state::AppState;

// Fixed window size matching NTSC square-pixel equivalent
// 640x480 = 480 visible lines (NTSC), 4:3 aspect ratio, square pixels
// Integer scales cleanly: 2x=1280x960, 3x=1920x1440
// This resolution matches what analog video synthesizers actually processed
const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;
const API_PORT: u16 = 3030;

struct App {
    state: Arc<RwLock<AppState>>,
    window: Option<Arc<Window>>,
    renderer: Option<renderer::Renderer>,
}

impl App {
    fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            window: None,
            renderer: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = WindowAttributes::default()
                .with_title("Device Workbench")
                .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_resizable(false);

            match event_loop.create_window(attrs) {
                Ok(window) => {
                    let window = Arc::new(window);
                    self.window = Some(window.clone());

                    // Initialize renderer
                    match pollster::block_on(renderer::Renderer::new(window, self.state.clone())) {
                        Ok(r) => {
                            info!("Renderer initialized");
                            self.renderer = Some(r);
                        }
                        Err(e) => {
                            tracing::error!("Failed to create renderer: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create window: {}", e);
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(ref mut renderer) = self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(ref mut renderer) = self.renderer {
                    if let Err(e) = renderer.render() {
                        tracing::error!("Render error: {}", e);
                    }
                }
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Device Workbench starting...");

    // Create shared state
    let state = Arc::new(RwLock::new(AppState::new()));

    // Spawn HTTP API server in background thread
    let api_state = state.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = api::run_server(api_state, API_PORT).await {
                tracing::error!("API server error: {}", e);
            }
        });
    });

    info!("API server running on http://localhost:{}", API_PORT);

    // Create event loop and run
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(state);
    event_loop.run_app(&mut app)?;

    Ok(())
}
