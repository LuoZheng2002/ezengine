Split the project into a small Fyrox-like workspace first.

Current milestone:
1. `ezengine-core` holds shared primitives.
2. `ezengine-ui` holds the tiny UI graph and button builders.
3. `ezengine-editor` owns the editor loop, renderer, and binary entry point.
4. UI nodes are stored directly as `Arc<AtomicRefCell<dyn UiNode>>`.
5. Button clicks execute subscribed `Fn()` callbacks immediately, and each subscription returns a `Subscription(usize)` that can be removed later.
6. UI rendering now uses `UiNodeDraw::draw()` and `DrawCommand::{PushVertex, Commit}`.
7. Next, extend the UI graph only as needed and keep the structure close to Fyrox.
