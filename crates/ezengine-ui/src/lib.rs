use std::{any::Any, collections::BTreeMap, sync::Arc};

use atomic_refcell::AtomicRefCell;
use ezengine_core::{Brush, Color, Point, Rect, Size};

pub type UiNodeRef = Arc<AtomicRefCell<dyn UiNode>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Subscription(pub usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex2D {
    pub position: Point,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    PushVertex(Vertex2D),
    Commit { brush: Brush },
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

    pub fn build(self, ui: &mut UserInterface) -> UiNodeRef {
        ui.add_button(self.widget.bounds)
    }
}

pub trait UiNode: Any + UiNodeDraw {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, bounds: Rect);
    fn children(&self) -> &[UiNodeRef];
    fn add_child(&mut self, child: UiNodeRef);
    fn hit_test(&self, point: Point) -> bool {
        self.bounds().contains(point)
    }
    fn set_hovered(&mut self, hovered: bool);
    fn set_pressed(&mut self, pressed: bool);
    fn click_callbacks(&self) -> Vec<Arc<dyn Fn()>>;
}

pub trait UiNodeDraw {
    fn draw(&self) -> Vec<DrawCommand>;
}

pub fn draw_rect(bounds: Rect, brush: Brush) -> Vec<DrawCommand> {
    let top_left = Point {
        x: bounds.origin.x,
        y: bounds.origin.y + bounds.size.height,
    };
    let top_right = Point {
        x: bounds.origin.x + bounds.size.width,
        y: bounds.origin.y + bounds.size.height,
    };
    let bottom_right = Point {
        x: bounds.origin.x + bounds.size.width,
        y: bounds.origin.y,
    };
    let bottom_left = Point {
        x: bounds.origin.x,
        y: bounds.origin.y,
    };

    vec![
        DrawCommand::PushVertex(Vertex2D {
            position: top_left,
        }),
        DrawCommand::PushVertex(Vertex2D {
            position: top_right,
        }),
        DrawCommand::PushVertex(Vertex2D {
            position: bottom_right,
        }),
        DrawCommand::PushVertex(Vertex2D {
            position: top_left,
        }),
        DrawCommand::PushVertex(Vertex2D {
            position: bottom_right,
        }),
        DrawCommand::PushVertex(Vertex2D {
            position: bottom_left,
        }),
        DrawCommand::Commit { brush },
    ]
}

impl<T: UiNodeDraw + ?Sized> UiNodeDraw for Box<T> {
    fn draw(&self) -> Vec<DrawCommand> {
        (**self).draw()
    }
}

pub struct UserInterface {
    _viewport: Size,
    root: UiNodeRef,
    cursor: Point,
    active_button: Option<UiNodeRef>,
}

impl UserInterface {
    pub fn new(viewport: Size) -> Self {
        let root = Arc::new(AtomicRefCell::new(PanelNode::new(Rect::new(
            0.0,
            0.0,
            viewport.width,
            viewport.height,
        ))));

        Self {
            _viewport: viewport,
            root,
            cursor: Point::default(),
            active_button: None,
        }
    }

    pub fn root(&self) -> UiNodeRef {
        Arc::clone(&self.root)
    }

    pub fn add_button(&mut self, bounds: Rect) -> UiNodeRef {
        let button = Arc::new(AtomicRefCell::new(ButtonNode::new(bounds)));
        self.root.borrow_mut().add_child(button.clone());
        button
    }

    pub fn subscribe_button<F>(&self, button: &UiNodeRef, on_click: F) -> Option<Subscription>
    where
        F: Fn() + 'static,
    {
        let mut borrowed = button.borrow_mut();
        borrowed
            .as_any_mut()
            .downcast_mut::<ButtonNode>()
            .map(|button| button.subscribe(on_click))
    }

    pub fn unsubscribe_button(&self, button: &UiNodeRef, subscription: Subscription) -> bool {
        let mut borrowed = button.borrow_mut();
        borrowed
            .as_any_mut()
            .downcast_mut::<ButtonNode>()
            .is_some_and(|button| button.unsubscribe(subscription))
    }

    pub fn set_viewport(&mut self, viewport: Size) {
        self._viewport = viewport;
        self.root.borrow_mut().set_bounds(Rect::new(0.0, 0.0, viewport.width, viewport.height));
    }

    pub fn handle_cursor_moved(&mut self, position: Point) {
        self.cursor = position;
        let hovered = self.find_button_at(position);
        self.update_hover_state(self.root(), hovered.as_ref());
    }

    pub fn handle_mouse_button(&mut self, pressed: bool) {
        if pressed {
            // Keep the active node so a matching release can trigger the callback.
            self.active_button = self.find_button_at(self.cursor);
            if let Some(button) = self.active_button.as_ref() {
                button.borrow_mut().set_pressed(true);
            }
            return;
        }

        if let Some(button) = self.active_button.take() {
            button.borrow_mut().set_pressed(false);

            // Click callbacks are copied out before execution so they can subscribe or unsubscribe safely.
            if self.find_button_at(self.cursor).is_some_and(|target| Arc::ptr_eq(&target, &button)) {
                for callback in button.borrow().click_callbacks() {
                    callback();
                }
            }
        }
    }

    pub fn draw_commands(&self) -> Vec<DrawCommand> {
        let mut commands = Vec::new();
        self.collect_draw_commands(self.root(), &mut commands);
        commands
    }

    fn find_button_at(&self, point: Point) -> Option<UiNodeRef> {
        self.find_button_at_in(self.root(), point)
    }

    fn find_button_at_in(&self, node: UiNodeRef, point: Point) -> Option<UiNodeRef> {
        {
            let borrowed = node.borrow();
            for child in borrowed.children().iter().rev() {
                if let Some(found) = self.find_button_at_in(child.clone(), point) {
                    return Some(found);
                }
            }

            if borrowed.hit_test(point) && borrowed.as_any().is::<ButtonNode>() {
                return Some(Arc::clone(&node));
            }
        }

        None
    }

    fn update_hover_state(&self, node: UiNodeRef, hovered: Option<&UiNodeRef>) {
        let mut borrowed = node.borrow_mut();
        // Only interactive nodes care about hover state; containers just forward traversal.
        let is_target = hovered.is_some_and(|target| Arc::ptr_eq(target, &node));
        if borrowed.as_any().is::<ButtonNode>() {
            borrowed.set_hovered(is_target);
        }

        let children = borrowed.children().to_vec();
        drop(borrowed);

        for child in children {
            self.update_hover_state(child, hovered);
        }
    }

    fn collect_draw_commands(&self, node: UiNodeRef, commands: &mut Vec<DrawCommand>) {
        let borrowed = node.borrow();
        commands.extend(borrowed.draw());
        let children = borrowed.children().to_vec();
        drop(borrowed);

        for child in children {
            self.collect_draw_commands(child, commands);
        }
    }
}

struct PanelNode {
    bounds: Rect,
    children: Vec<UiNodeRef>,
}

impl PanelNode {
    fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            children: Vec::new(),
        }
    }
}

impl UiNode for PanelNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn children(&self) -> &[UiNodeRef] {
        &self.children
    }

    fn add_child(&mut self, child: UiNodeRef) {
        self.children.push(child);
    }

    fn set_hovered(&mut self, _hovered: bool) {}

    fn set_pressed(&mut self, _pressed: bool) {}

    fn click_callbacks(&self) -> Vec<Arc<dyn Fn()>> {
        Vec::new()
    }
}

impl UiNodeDraw for PanelNode {
    fn draw(&self) -> Vec<DrawCommand> {
        Vec::new()
    }
}

struct ButtonNode {
    bounds: Rect,
    children: Vec<UiNodeRef>,
    hovered: bool,
    pressed: bool,
    base_color: Color,
    hover_color: Color,
    pressed_color: Color,
    next_subscription_id: usize,
    subscriptions: BTreeMap<usize, Arc<dyn Fn()>>,
}

impl ButtonNode {
    fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            children: Vec::new(),
            hovered: false,
            pressed: false,
            base_color: Color::rgb(0.20, 0.24, 0.30),
            hover_color: Color::rgb(0.28, 0.40, 0.62),
            pressed_color: Color::rgb(0.15, 0.53, 0.84),
            next_subscription_id: 0,
            subscriptions: BTreeMap::new(),
        }
    }

    fn subscribe<F>(&mut self, on_click: F) -> Subscription
    where
        F: Fn() + 'static,
    {
        let subscription = Subscription(self.next_subscription_id);
        self.next_subscription_id += 1;
        let callback: Arc<dyn Fn()> = Arc::new(on_click);
        self.subscriptions.insert(subscription.0, callback);
        subscription
    }

    fn unsubscribe(&mut self, subscription: Subscription) -> bool {
        self.subscriptions.remove(&subscription.0).is_some()
    }

    fn color(&self) -> Color {
        // Keep the button visuals simple: pressed wins over hover, hover wins over idle.
        if self.pressed {
            self.pressed_color
        } else if self.hovered {
            self.hover_color
        } else {
            self.base_color
        }
    }
}

impl UiNode for ButtonNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn children(&self) -> &[UiNodeRef] {
        &self.children
    }

    fn add_child(&mut self, child: UiNodeRef) {
        self.children.push(child);
    }

    fn set_hovered(&mut self, hovered: bool) {
        self.hovered = hovered;
    }

    fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    fn click_callbacks(&self) -> Vec<Arc<dyn Fn()>> {
        self.subscriptions.values().cloned().collect()
    }
}

impl UiNodeDraw for ButtonNode {
    fn draw(&self) -> Vec<DrawCommand> {
        draw_rect(self.bounds, Brush::solid(self.color()))
    }
}
