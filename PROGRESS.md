Started the editor bootstrap work.

Implemented:
1. `Editor::run()` entry point.
2. `src/bin/bin_test_editor.rs` as the named test binary.
3. A minimal `winit` + `wgpu` renderer that draws a gradient triangle.
4. Verified with `cargo check`.
5. Split the codebase into `ezengine-core`, `ezengine-ui`, and `ezengine-editor`.
6. Added a small UI graph with a button that reacts to hover and click.
7. Switched the UI to direct `Arc<AtomicRefCell<dyn UiNode>>` references and callback-based button clicks.
8. Added dynamic button subscription and unsubscription via `Subscription(usize)`.
9. Switched UI rendering to a command stream with `UiNodeDraw` and `DrawCommand::{PushVertex, Commit}`.

Still to do:
1. Add the rest of the editor systems as `// todo`-marked scaffolding.
2. Expand the UI graph only when a new editor control needs it.
