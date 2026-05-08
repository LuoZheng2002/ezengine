This project is to build a minimal replication of the Fyrox game engine with only the core features. It does not need to be polished.

The cloned Fyrox repository for reference is at C:\Users\luo20\Desktop\fyrox_projects\MyProject.

The major difference we target is:
1. Do not care about performance as long as the asymptotic complexity is the same. Specifically, do not implement a complicated object pool system, but use `Arc<AtomicRefCell<dyn MyTrait>>` everywhere where we want to store a collection of objects (like scene nodes) and / or we want to keep a reference in a user script.
2. Avoid writing custom macros.
3. Use single-threaded execution everywhere (but still keep all reference counters as Arc and interior mutability as AtomicRefCell).
4. Use wgpu as the only graphics backend, so we do not do polymorphism on graphics backends.
5. Target only Windows platform, but do not use Windows-specific crates.

The progress is in PROGRESS.md.
The current task is in CURRENT_TASK.md.
You should update both files while updating the code.