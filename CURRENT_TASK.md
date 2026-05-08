Start from replicating `Editor::run()`.

Milestone 1 implemented:
1. `src/bin/bin_test_editor.rs` calls `Editor::run()` and boots a `winit` event loop plus a `wgpu` graphics context.
2. The editor renders a simple color-gradient triangle.
3. The code is still intentionally small so we can split it into `ezengine-*` crates later.
4. Everything outside this milestone should remain marked with `// todo` in the code.
