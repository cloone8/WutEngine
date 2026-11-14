# Principles and constraints

The rules that apply across every part of the engine, and the hard technical constraints the priorities are weighed
within.

## Design principles

- **Simple for the end user, above all.** Installing and using the engine must be easy: game creators aren't
  necessarily programmers. The user-facing API is simple, with little magic.
  Why: the audience includes non-programmers, and jam teams must be productive within an afternoon.
- **"No magic" is for Rust; scripts and the editor may help non-programmers.** The engine's internals and its Rust API
  have no hidden registration and no behaviour that only works through a macro or a naming convention. In the
  scripting and editor layer, a bit of magic that helps non-programmers is fine (for example, custom shader-graph nodes
  are assets the editor finds by itself). Boundary: what the editor finds this way must still work without the editor
  (a graph references a node asset by its UUID, so the CLI compiles it).
  Why: Rust programmers get the engine's internals directly; scripting is for fast iteration by non-programmers.
- **The editor is the main way to make a game, but not the only one.** A game can be made entirely in Rust, with the
  CLI tools to create and convert assets. Anything the editor does with assets stays possible without it.
- **The engine is the platform for its own tooling: GUI tools are WutEngine games.** The editor, and any future tool
  with a GUI, runs on the engine runtime and draws itself with it (egui through the engine's integration), never on a
  separate app framework. When a tool needs something the engine lacks, it is added to the engine.
  Why: tooling drives improvement, testing and versatility of the core.
- **Core engine operations are plain functions over global state.** Loading a mesh, switching scenes, reading config:
  a free function you can call, backed by module-level global state (a static per service,
  [Lifecycle and memory](../core/lifecycle.md)). No context argument is passed around.
  Why: passing every engine system around as an argument was too verbose; the globals are deliberate.
- **ECS underneath, objects for the editor and scripts; engine services are not ECS.** Game objects live in an ECS,
  and the editor and scripts see them as objects ([Object model](../world/object_model.md)). Engine services (graphics
  device, asset server, configuration, time) are not ECS resources or events.
  Why: the explicit anti-goal is a design where the user API looks simple but the engine code is unreadable.
- **Engine internals stay readable.** Direct code over layers of traits, generics and macros whose only job is to make
  the user-facing API look neat. Why: the same anti-goal, a simple-looking API over unreadable engine code.
- **Stable identity for serialized types.** Components and asset types carry hardcoded UUIDs that project files
  reference; a new type gets a freshly generated v4 UUID. Keeping existing ones unchanged so old project files load
  is not required during 0.x (below).
  Why: project files reference types by these UUIDs, not by their names.
- **Runtime data is separate from serialized data.** What the running game uses is chosen for the runtime (speed,
  memory); what's stored on disk is chosen for the editor, version control and loading. They are converted at load
  and save, never forced to be the same. Examples: a full UUID per entity in scene files while the runtime uses a fast
  ID; f32 positions at runtime but absolute f64 in the editor and scripts
  ([Game size and precision](game_size.md)); JSON project files but runtime-ready binary packs.
  Why: each side can then be optimized for its own job.
- **Simple defaults, advanced APIs exposed.** The obvious path is simple, so a small team or a jam is productive
  immediately; the complex APIs (async loading, streaming, fine control) are exposed too, not hidden. "Simple"
  describes what the developer sees, not how the engine implements it: a scene switch feels synchronous to the script
  while the engine loads asynchronously and never blocks a frame.
  Why: larger games have specialised programmers who need the advanced APIs.
- **Good use of many cores.** Engine work, tools (the editor included) and Rust code are designed to run in parallel,
  and parallel is the easy path for Rust programmers. Script users never deal with threads. Parallel work always gives
  the same result as running it on one thread, so the single-threaded web behaves identically
  ([Time and the frame](../core/frame.md)). Every subsystem is designed for it, and APIs prefer shapes that
  parallelize (batch operations, data in arrays; per-object callbacks only where the engine can spread them).
  Why: results must not differ between desktop and the web.

## Version 0 stays flexible

- **No project-file compatibility at all while the engine is at major version 0.** Why: the design can still change
  freely.
- **The engine stays at 0 until one full working game has shipped with it**, even a small one; tech demos and
  Pong-like test projects don't count.
- **From 1.0:** projects keep working within a major version (small changes through serde's `alias` and `default`,
  larger ones through versioned migrations run when an older project is opened, after a backup or commit warning).
  Across major versions nothing is promised; conversion tools are welcome but not critical. A project saved by a newer
  engine is refused by an older one.

## Scripting language

Game scripts are TypeScript, compiled to JavaScript and run on the boa JavaScript engine (no JIT)
([Scripting](../scripting/README.md)).

## Technical constraints

Hard constraints, not traded off against the priority order.

| | Required | Preferred | Kept possible | Not needed |
|---|---|---|---|---|
| Runtime | Windows, Linux, macOS, web (`wasm32-unknown-unknown`) | Android, iOS | Consoles | — |
| Editor | Windows, Linux, macOS | Web (a bonus, not designed for) | — | Everything else |

- **The runtime doesn't exclude a platform when that can be avoided.** Never emscripten on the web: winit and the Rust
  ecosystem use `wasm32-unknown-unknown`.
- **Rust 1.100 or later.** Why: it stabilises the `Allocator` trait with `Vec<T, A>` and `Box<T, A>`, which the
  memory design relies on ([Lifecycle and memory](../core/lifecycle.md)).
- **Graphics through `wgpu`, no own hardware abstraction layer.** Why: cross-platform coverage.
- **WebGPU only, no WebGL2 or GLES fallback.** Players whose browser lacks WebGPU use the native download.
  Why: a WebGL2 fallback would cap the renderer at wgpu's downlevel limits (no compute, among others), which cuts into
  modern rendering features and what there is to learn.
- **The runtime works single-threaded on the web; threads are an optimization, not a requirement.** Wasm threads may
  later be an optional accelerator, enabled only when the page is cross-origin isolated. The scheduler and job system
  therefore need a sequential path that never blocks the main thread ([Jobs and async](../core/jobs.md)).
  Why: wasm threads still need a nightly toolchain (`-Zbuild-std`), and they need COOP/COEP headers that game hosts
  handle poorly.
- **Web and mobile share requirements:** no plain filesystem for assets (APK, browser fetch), an app lifecycle
  (suspend and resume; graphics initializes after the app is resumed), touch input. Designing for the web covers most
  of mobile.
- **No JIT.** Why: iOS forbids it for apps, and keeping consoles possible requires it too.
- **Consoles kept possible.** They can't be designed for without SDKs, so "kept possible" means the same rules as web
  and iOS (no OS assumptions, no JIT, no plain filesystem), plus platform code (windowing, input, files) kept behind
  the engine's own layer so it can be swapped. Runtime code has no hard dependency on a desktop OS. wgpu on a console
  would need someone with SDK access to port a backend.
