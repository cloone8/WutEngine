# Runtime crates

`wutengine` is the crate games depend on. It re-exports and drives the subsystem crates next to it
(`wutengine_graphics`, `_input`, `_physics`, `_audio`, `_time`, `_config`, `_task`, `_event`, `_logger`, `_math`,
`_egui`, `_development_overlay`). Its features are `phys2d`, `phys3d` (both default), `profiling`,
`development_overlay` and `asset_importers`.

`wutengine_puffin_egui` is a vendored fork of `puffin_egui` (its own licences and changelog). Leave it alone unless the
task is about it; `rustfmt.toml` skips it.

## How the engine runs

- `wutengine::runtime::start` (`wutengine/src/runtime/init.rs`) runs once per process, on the main thread. It
  initializes the subsystems in a fixed order, then runs the winit event loop (`runtime/winit_app.rs`). Graphics is
  only initialized in winit's `resumed`.
- A frame (`Runtime::run_frame` in `runtime/mod.rs`) runs the systems per `Phase` (`system/mod.rs`): fixed updates
  and physics, `Update`, `LateUpdate`, `PreRender`, then rendering and present.
- Systems are closures registered per phase (`SystemManifest::add_system`). The scheduler (`system/scheduler.rs`)
  runs systems whose borrows don't conflict in parallel on rayon. Entity spawns, component additions and destroys are
  deferred and applied between phases (`entity::process_changes`).
- Rendering: components queue draw commands in `PreRender`; render passes are components (`CameraRenderPass`,
  `OverlayRenderPass`); each camera renders to a texture on its own command encoder, then gets blitted to the window.

## Global state

Engine services are globals, on purpose (root `AGENTS.md`, Design principles). Keep them safe:

- **Initialization order.** `InitOnce` statics are filled during `start`. A `new_checked()` one panics when read
  before `init`. An `unsafe new_unchecked()` one only checks in debug builds and is undefined behaviour in release.
  Don't read a global from code that can run before its `init` (another global's `init`, a `const`/`static`
  initializer, code called before `resumed` for graphics). Prefer `new_checked()` for new globals.
- **Main thread.** `InitOnce::init` asserts the main thread by default, and some state is `MainThreadOnly`
  (`wutengine_util`). Code running inside systems runs on rayon workers: reach main-thread-only state through
  `send_to_main_thread` (`wutengine/src/runtime/mod.rs`), never directly.
- **Locks.** Globals behind `RwLock`/`DashMap` are used from parallel systems. Don't hold a guard across a call that
  might lock the same global again, or across a phase or frame boundary.

## The web target

Everything in `runtime/` (and what it depends on in `util/` and `asset/`) must keep working on
`wasm32-unknown-unknown`. The browser has no filesystem, no blocking on the main thread, and threads only with extra
setup. So in runtime code:

- No unconditional `std::fs`, `std::thread`, `std::time::Instant`/`SystemTime`, blocking waits (`pollster::block_on`,
  spin-sleeping) or OS-specific APIs. Put them behind `cfg_select!` with a web branch, or keep them out of runtime crates.
- No JIT or runtime code generation.

Existing code already breaks these rules; it has no wasm build yet. Known cases: the pipeline cache saved to disk and
`pollster::block_on` for the adapter and device (`wutengine_graphics/src/init.rs`), `pollster::block_on` in shader
compilation (`shader/compile.rs`), the thread pools, `block_on` and CPU detection in `wutengine_task`, and the
`spin_sleep` frame pacer (`wutengine/src/window/pacer.rs`). Don't add to that list; when you touch one of them, mention
that it blocks the web target.

## Embedded files

Built-in shaders, the default texture and the window icon are embedded with `include_str!` /
`include_bytes!` (`wutengine/src/builtins/shaders/`, `wutengine_egui/src/egui.wgsl`,
`wutengine_graphics/src/default_texture.png`). A grep for callers of a Rust item doesn't find these: search for the
file name before moving or deleting one.
