Split the project into a small Fyrox-like workspace first.

Current milestone:
1. `ezengine-core` holds shared primitives.
2. `ezengine-ui` holds the tiny UI graph and button builders.
3. `ezengine-editor` owns the editor loop, renderer, and binary entry point.
4. The editor now has a simple button with hover and click feedback.
5. Next, extend the UI graph only as needed and keep the structure close to Fyrox.
