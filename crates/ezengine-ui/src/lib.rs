use ezengine_core::{Color, Point, Rect, Size};

pub type UiHandle = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiMessageKind {
    HoverChanged(bool),
    Clicked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiMessage {
    pub handle: UiHandle,
    pub kind: UiMessageKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonVisual {
    pub bounds: Rect,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct WidgetBuilder {
    bounds: Rect,
}

impl WidgetBuilder {
    pub fn new() -> Self {
        Self {
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn with_bounds(mut self, bounds: Rect) -> Self {
        self.bounds = bounds;
        self
    }
}

impl Default for WidgetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct ButtonBuilder {
    widget: WidgetBuilder,
}

impl ButtonBuilder {
    pub fn new(widget: WidgetBuilder) -> Self {
        Self { widget }
    }

    pub fn build(self, ui: &mut UserInterface) -> UiHandle {
        ui.add_button(self.widget.bounds)
    }
}

#[derive(Clone, Copy, Debug)]
struct ButtonNode {
    hovered: bool,
    pressed: bool,
    base_color: Color,
    hover_color: Color,
    pressed_color: Color,
}

impl ButtonNode {
    fn new() -> Self {
        Self {
            hovered: false,
            pressed: false,
            base_color: Color::rgb(0.20, 0.24, 0.30),
            hover_color: Color::rgb(0.28, 0.40, 0.62),
            pressed_color: Color::rgb(0.15, 0.53, 0.84),
        }
    }

    fn color(&self) -> Color {
        if self.pressed {
            self.pressed_color
        } else if self.hovered {
            self.hover_color
        } else {
            self.base_color
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum NodeKind {
    Root,
    Button(ButtonNode),
}

#[derive(Clone, Debug)]
struct Node {
    bounds: Rect,
    _parent: Option<UiHandle>,
    children: Vec<UiHandle>,
    kind: NodeKind,
}

pub struct UserInterface {
    _viewport: Size,
    nodes: Vec<Node>,
    root: UiHandle,
    messages: Vec<UiMessage>,
    cursor: Point,
    active_button: Option<UiHandle>,
}

impl UserInterface {
    pub fn new(viewport: Size) -> Self {
        let root = Node {
            bounds: Rect::new(0.0, 0.0, viewport.width, viewport.height),
            _parent: None,
            children: Vec::new(),
            kind: NodeKind::Root,
        };

        Self {
            _viewport: viewport,
            nodes: vec![root],
            root: 0,
            messages: Vec::new(),
            cursor: Point::default(),
            active_button: None,
        }
    }

    pub fn root(&self) -> UiHandle {
        self.root
    }

    pub fn add_button(&mut self, bounds: Rect) -> UiHandle {
        let handle = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            _parent: Some(self.root),
            children: Vec::new(),
            kind: NodeKind::Button(ButtonNode::new()),
        });
        self.nodes[self.root].children.push(handle);
        handle
    }

    pub fn set_viewport(&mut self, viewport: Size) {
        self._viewport = viewport;
        self.nodes[self.root].bounds = Rect::new(0.0, 0.0, viewport.width, viewport.height);
    }

    pub fn handle_cursor_moved(&mut self, position: Point) {
        self.cursor = position;
        let button = self.find_button_at(position);
        for handle in 1..self.nodes.len() {
            if let Some(node) = self.nodes.get_mut(handle) {
                if let NodeKind::Button(state) = &mut node.kind {
                    let hovered = Some(handle) == button;
                    if state.hovered != hovered {
                        state.hovered = hovered;
                        self.messages.push(UiMessage {
                            handle,
                            kind: UiMessageKind::HoverChanged(hovered),
                        });
                    }
                }
            }
        }
    }

    pub fn handle_mouse_button(&mut self, pressed: bool) {
        if pressed {
            self.active_button = self.find_button_at(self.cursor);
            if let Some(handle) = self.active_button {
                if let Some(node) = self.nodes.get_mut(handle) {
                    if let NodeKind::Button(state) = &mut node.kind {
                        state.pressed = true;
                    }
                }
            }
            return;
        }

        if let Some(handle) = self.active_button.take() {
            if let Some(node) = self.nodes.get_mut(handle) {
                if let NodeKind::Button(state) = &mut node.kind {
                    state.pressed = false;
                }
            }

            if self.find_button_at(self.cursor) == Some(handle) {
                self.messages.push(UiMessage {
                    handle,
                    kind: UiMessageKind::Clicked,
                });
            }
        }
    }

    pub fn drain_messages(&mut self) -> impl Iterator<Item = UiMessage> + '_ {
        self.messages.drain(..)
    }

    pub fn button_visual(&self, handle: UiHandle) -> Option<ButtonVisual> {
        self.nodes.get(handle).and_then(|node| match node.kind {
            NodeKind::Button(state) => Some(ButtonVisual {
                bounds: node.bounds,
                color: state.color(),
            }),
            NodeKind::Root => None,
        })
    }

    fn find_button_at(&self, point: Point) -> Option<UiHandle> {
        for (handle, node) in self.nodes.iter().enumerate().rev() {
            if matches!(node.kind, NodeKind::Button(_)) && node.bounds.contains(point) {
                return Some(handle);
            }
        }
        None
    }
}
