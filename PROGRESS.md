Started the editor bootstrap work.

Implemented:
1. `Editor::run()` entry point.
2. `src/bin/bin_test_editor.rs` as the named test binary.
3. A minimal `winit` + `wgpu` renderer that draws a gradient triangle.
4. Verified with `cargo check`.

Still to do:
1. Split the project into the Fyrox-style workspace layout with `ezengine-*` crates.
2. Add the rest of the editor systems as `// todo`-marked scaffolding.
