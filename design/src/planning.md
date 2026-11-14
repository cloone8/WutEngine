# Planning

The order to build the engine in, from an empty workspace to jam-ready and beyond. Read it top to bottom: each step
says what to build, which chapters it implements, and when it is done. A step may start before the previous one is
complete when it doesn't depend on it.

Some orderings are decided and marked **Fixed**. The rest is a proposed order around them: change it freely, as with
any decision in this book.

Fixed orderings that run through several steps:
- **The web is a first-class target from the start.** Proposed on top of that: every step is done when it works in
  a browser too, not only on desktop. Why: jam games are mostly played in the browser, and web constraints (no
  threads, no blocking, grow-only memory) are expensive to retrofit.
- **Debugging basics early,** because they help build everything else ([Profiling and debugging
  tools](editor/debug_tools.md)).
- **The renderer follows its feature order,** steps 0 to 8 ([Renderer](rendering/README.md)). The first game sets
  what comes first; 2D comes in early, as small additions next to the 3D work.
- **Stand-in crates may ship.** Replacing a stand-in with an own version is learning work, scheduled by interest. It
  holds up neither the first game nor 1.0 ([Why the engine exists](goals/why.md)).

## 1. Workspace

- The cargo workspace at version 0.7.0, MPL-2.0, Rust 1.100 or later pinned, strict workspace lints, the formatter
  and clippy configuration, `.gitignore`, and `cargo xtask` as the internal build tool ([Principles and
  constraints](goals/principles.md), [Building and shipping](editor/shipping.md)).
- Logging and profiling scopes (puffin) from the first crate.

Done when an empty engine crate builds for desktop and `wasm32-unknown-unknown`, and clippy is clean.

## 2. Start-up, a window, shut-down

- Start-up as one `async` function, a static per service, the winit event loop, one window; on the web, a canvas and
  the loader library's start function ([Lifecycle and memory](core/lifecycle.md), [Windowing and
  display](systems/windowing.md), [Building and shipping](editor/shipping.md)).
- Shut-down as flush and exit, quit requested and quitting, `panic = "abort"` with a panic hook, `fatal_error`, and
  the crash message ([Error handling](core/errors.md)).
- Headless as an additive build feature. **Fixed:** from the start ([Lifecycle and memory](core/lifecycle.md)).
- The typed configuration: sections registered in the entry point, the layered files, the command line, and the
  engine's early sections ([Configuration](core/configuration.md)).

Done when a window opens and closes cleanly on desktop and in a browser, configured from a file and the command line.

## 3. Time, frame and jobs

- The frame's phases, the fixed-update loop with the 0.25 s clamp, pause and time scale ([Time and the
  frame](core/frame.md)). Frame timing: **Fixed:** the raw measured delta first; snapping and presentation timestamps
  come later (steps 15 and Later).
- The job system: `Task<T>`, the main-thread executor (`run_async`) also driving desktop start-up, `compute` on rayon,
  budgeted work on the web ([Jobs and async](core/jobs.md)).
- `Name`, the conventions' math types, and `WorldPos` with the f32/f64 build switch ([Conventions and
  names](core/conventions.md), [Game size and precision](goals/game_size.md)).
- Frame pacing: vsync, acquiring the surface at the end of the frame, the frame limiter ([Time and the
  frame](core/frame.md)).
- Memory attribution by logical counters ([Lifecycle and memory](core/lifecycle.md)).
- The development overlay with the puffin flamegraph, and the development console over settings ([Profiling and
  debugging tools](editor/debug_tools.md), [Configuration](core/configuration.md)).

Done when an empty game loop runs at a steady pace with a profiler overlay, on desktop and the web.

## 4. The world in Rust

- The object model on hecs: entities, components with UUIDs and explicit registration, systems with declared access,
  the scheduler (non-conflicting systems in parallel, conflicting ones in registration order), commands applied after
  each group, and the schedule view ([Object model](world/object_model.md), [Time and the frame](core/frame.md)).
- The transform hierarchy and its propagation pass; transform-less entities ([Object model](world/object_model.md)).
- Reflection: serde values plus a derived schema; decide the derive crate here ([Reflection](world/reflection.md)).

Done when Rust systems move a hierarchy of entities in parallel with the same result as on one thread, and a
component round-trips through its reflected form.

## 5. Renderer step 0: the foundation

- **Fixed:** the device with every available feature, checked at runtime; the render graph with injection points;
  views; the retained GPU scene; shared mesh buffers; `weshc` with shading-model wrapping and pass variants; debug
  labels; the `rgba16float` target with tonemapper None and sRGB output; compute passes and compute shaders from the
  start. Unlit on top ([Renderer](rendering/README.md), [Renderer architecture](rendering/architecture.md), [Shaders
  and materials](rendering/shaders.md)).
- The camera component ([Input and camera](systems/input_camera.md)).
- Owed at this step: an explanation of reversed Z, the depth convention ([Conventions and
  names](core/conventions.md)).

Done when Rust code draws meshes with unlit materials through a camera, on desktop and the web.

## 6. Assets, project files and the CLI tools

- The project format: JSON files, one settings file per asset, and the regenerable import cache
  ([Assets and project files](world/assets.md)).
- One format library for the tools and the player: `weimport`, `weassetconv` and `weshc` on it.
- `Asset<T>` and `AssetRef<T>`, loading as a task, placeholders, freeing at frame end. The hot-reload API shape goes
  in now; reloading itself comes later. **Fixed:** "API first" ([Assets and project files](world/assets.md)).
- Packs and the manifest, loaded on desktop and the web, with texture formats chosen per target at build time and
  variants in the web build ([Building and shipping](editor/shipping.md)).

Done when the CLI tools turn a glTF and a PNG into a pack that a Rust game loads and draws in a browser.

## 7. Scenes

- The scene asset, instances with scene properties, one root per scene, `load`, `unload` and `switch`, the start
  scene, global entities, and finding by type and tag ([Scenes](world/scenes.md)).

Done when a game made entirely in Rust switches between two scenes loaded from packs, without a frame hitch.

## 8. Input and windowing

- Input actions in one input asset, the per-tick input record, action sets and rebinding, keys by physical position,
  gamepads through gilrs, and the mouse ([Input and camera](systems/input_camera.md)).
- The rest of windowing: multiple windows, cursor modes, focus, IME, clipboard and file drop ([Windowing and
  display](systems/windowing.md)).

Done when a first-person camera walks around a scene with keyboard and mouse or a gamepad, on desktop and the web.

## 9. Scripting

- boa with TypeScript compiled by `tsc`, script components with exported properties, the callbacks, async waits and
  the settle point ([Scripting](scripting/README.md), [Script API](scripting/api.md)).
- Signals ([Signals](scripting/signals.md)).
- Script errors that never crash the engine, the loop limit, and the generated `.d.ts` ([Error
  handling](core/errors.md), [Documentation](editor/documentation.md)).

Done when the walking camera from step 8 is a TypeScript script, with the same behaviour on desktop and the web.

## 10. Renderer steps 1 and 2: unlit complete, 2D basics, the first-game kit

- **Fixed (step 1):** alpha clip and blending with a sorted transparent pass, vertex colours, a sky, sprites with an
  explicit 2D sort order, pixel-art texture settings, and the compute API outside the frame.
- **Fixed (step 2):** fullscreen post materials at injection points, a low-resolution view upscaled with nearest
  filtering, distance fog, custom interpolators, and vertex-stage access to the clip-space position
  ([Renderer](rendering/README.md)).

Done when the PSX look runs as a user shading model plus post materials, and a small 2D scene works too.

## 11. Physics and audio

- Rigid bodies and colliders on rapier, queries, collision layers, physics materials and surface types, the four
  callbacks, and the kinematic character controller ([Physics](systems/physics.md)).
- The engine's own mixer (desktop output thread, web audio worklet), `Sound` assets, sources and one-shots with
  playbacks, the bus tree, the listener, and the web's audio unlock ([Audio](systems/audio.md)).

Done when a scripted player walks with the character controller, collides, and hears positional sound in a browser.

## 12. In-game UI and text

- UI screens of UI entities with CSS-shaped layout, UI units, input through the UI action set, full Unicode text with
  fonts and fallback, the localizable text type, and accessibility semantics ([In-game UI and
  text](systems/ui_text.md)).

Done when the game has a main menu and in-game text that work with mouse, keyboard and gamepad.

## 13. Milestone: the first game

The small 3D project with a PSX look ([Why the engine exists](goals/why.md)), made in code and scripts with the CLI
tools, before the editor exists. Saves come in now if the game needs them ([Save games and
storage](world/save_storage.md)).

Done when the game is finished and playable on desktop and the web.

## 14. The editor

- The editor as a WutEngine game on egui: projects, the asset browser and import, the scene view, and the inspector
  built from reflection ([Editor](editor/README.md)).
- Every edit a change record through one edit API, undo and redo per document, atomic save, copy and paste.
- Play mode in its own process over the editor link, with live editing. **Fixed:** the play-mode window starts as a
  separate window; embedding it in the editor comes later.
- Building and shipping from the editor: desktop builds and the web export with its templates ([Building and
  shipping](editor/shipping.md)).
- Editor extensions in TypeScript and Rust; packages ([Packages](world/packages.md)).

Done when the first game's scenes can be opened, edited, played and built from the editor.

## 15. Jam-ready

- The install flow: a prebuilt editor download that detects what's missing and guides the toolchain install
  ([Why the engine exists](goals/why.md)).
- The template project with the drop-in first-person player, and a getting-started guide in the manual
  ([Documentation](editor/documentation.md)).
- **Fixed:** crash recovery through the change journal, required before the engine is called jam-ready
  ([Editor](editor/README.md)).
- Hot reload of assets and scripts in play mode ([Assets and project files](world/assets.md),
  [Scripting](scripting/README.md)).
- Frame-time snapping. **Fixed:** after the raw delta, before presentation timestamps ([Time and the
  frame](core/frame.md)).
- Save games and storage, if not done for the first game ([Save games and storage](world/save_storage.md)).

Done when someone who has never seen the engine makes a small game with it in an afternoon, and it runs in a browser.

## 16. After jam-ready

- The rest of the renderer's feature order, in its **fixed** order (placing it after jam-ready is the proposal):
  simple lighting (3), shadows (4), many lights through clustered forward (5), PBR (6), post effects (7), GPU-driven
  rendering (8) ([Renderer](rendering/README.md)).
- Interpolation of displayed poses, as an addition to render preparation ([Time and the frame](core/frame.md)).
- The render graph's tooling as the renderer grows: graph view, texture viewer, GPU time per pass, frame debugger
  ([Renderer architecture](rendering/architecture.md)).
- The shader graph and visual scripting on the shared node editor widget ([Shaders and
  materials](rendering/shaders.md), [Visual scripting](scripting/visual_scripting.md)).
- The "static" marking. **Fixed:** built together with its first user, such as baked lighting or navigation
  ([Conventions and names](core/conventions.md)).
- Features from [Later](later.md), by what the next game needs or by interest; replacing stand-ins by interest.
- **1.0** follows the first complete game shipped with the engine ([Principles and constraints](goals/principles.md)).

## Where a different order may suit you

- **The first game before or after the editor (step 13).** Before means sooner, and the engine's Rust-only path gets
  tested. After means the first game also tests the editor.
- **Scripting before or after the renderer's steps 1 and 2.** It's proposed earlier because the script API shapes
  how everything after it is used.
- **UI (step 12)** is the decision the design is least sure about. Building only what the first game needs, then
  revisiting the design, may be better than building it all at once.
- **Shadows and simple lighting** might be wanted before the first game, if the PSX look turns out to need them.
