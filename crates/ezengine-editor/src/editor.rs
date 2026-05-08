use std::sync::Arc;

use ezengine_core::{Point, Rect, Size};
use ezengine_ui::{ButtonBuilder, UiNodeRef, UserInterface, WidgetBuilder};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::render::{FrameResult, Renderer};

pub struct Editor;

impl Editor {
    pub fn run() {
        // todo: split the editor shell into a dedicated dock/layout system.
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
    ui: Option<UserInterface>,
    button: Option<UiNodeRef>,
    button_subscription: Option<ezengine_ui::Subscription>,
}

impl EditorApp {
    fn button_rect(&self) -> Rect {
        Rect::new(40.0, 40.0, 180.0, 54.0)
    }
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
        let size = window.inner_size();

        let mut ui = UserInterface::new(Size {
            width: size.width as f32,
            height: size.height as f32,
        });
        // Keep a direct node reference here so the editor can render the button without a handle table.
        let button = ButtonBuilder::new(WidgetBuilder::new().with_bounds(self.button_rect())).build(
            &mut ui,
        );

        let button_subscription = ui
            .subscribe_button(&button, || {
                println!("button clicked");
            })
            .expect("button subscription failed");

        let renderer = pollster::block_on(Renderer::new(window.clone()))
            .expect("failed to initialize the wgpu renderer");

        self.window_id = Some(window.id());
        self.renderer = Some(renderer);
        self.ui = Some(ui);
        self.button = Some(button);
        self.button_subscription = Some(button_subscription);
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
                // The subscription can be removed later if the editor needs to disconnect the callback.
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
                if let Some(ui) = self.ui.as_mut() {
                    ui.set_viewport(Size {
                        width: size.width as f32,
                        height: size.height as f32,
                    });
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.handle_cursor_moved(Point {
                        x: position.x as f32,
                        y: position.y as f32,
                    });
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.handle_mouse_button(state == ElementState::Pressed);
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(renderer), Some(ui)) = (self.renderer.as_mut(), self.ui.as_ref())
                {
                    let draw_commands = ui.draw_commands();
                    // The renderer consumes the UI's command stream every frame.
                    match renderer.render(&draw_commands) {
                        FrameResult::Presented => {}
                        FrameResult::NeedsResize => {
                            if let Some(window) = self.window.as_ref() {
                                renderer.resize(window.inner_size());
                            }
                        }
                        FrameResult::Skipped => {}
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
