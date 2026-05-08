use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::render::Renderer;

pub struct Editor;

impl Editor {
    pub fn run() {
        // todo: move the editor bootstrap into a dedicated ezengine-editor crate.
        let event_loop = EventLoop::new().expect("failed to create the event loop");
        let mut app = EditorApp::default();
        event_loop.run_app(&mut app).expect("editor event loop failed");
    }
}

#[derive(Default)]
struct EditorApp {
    window: Option<Arc<Window>>,
    window_id: Option<WindowId>,
    renderer: Option<Renderer>,
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("ezengine editor")
                    .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720)),
            )
            .expect("failed to create editor window");

        let window = Arc::new(window);
        let renderer = pollster::block_on(Renderer::new(window.clone()))
            .expect("failed to initialize the wgpu renderer");

        self.window_id = Some(window.id());
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(expected_id) = self.window_id else {
            return;
        };

        if expected_id != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    match renderer.render() {
                        Ok(crate::render::FrameResult::Presented) => {}
                        Ok(crate::render::FrameResult::NeedsResize) => {
                            if let Some(window) = self.window.as_ref() {
                                renderer.resize(window.inner_size());
                            }
                        }
                        Ok(crate::render::FrameResult::Skipped) => {}
                        Err(_) => {
                            event_loop.exit();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}
