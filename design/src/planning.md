# Planning

The order to build the engine in, from an empty workspace to jam-ready and beyond. Read it top to bottom: each step
says what to build, which chapters it implements, the demos that show it working, and when it is done. A step may
start before the previous one is complete when it doesn't depend on it.

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

## Demos

Every step is cut into checkpoints, and each checkpoint ends in a demo: a small program that shows the new work
running. Proposed. Why: months of engine code with nothing to run hide mistakes until they are expensive, and are
less fun.

- **A demo is a program in the repository's `demos/` folder,** run with one `cargo xtask` command on desktop, or
  built and served locally for a browser with a flag. On the web, log output is the browser console.
- **A checkpoint is done when its demo runs on desktop and in a browser,** like the step it belongs to, except
  where the feature doesn't exist on the web (more windows, headless, the CLI tools, the editor).
- **Demos that fail on purpose** (a crash, a broken script) do it only when a demo setting asks, so the smoke run
  below stays clean.
- **Demos are kept working:** a step that breaks an earlier demo fixes it. One `cargo xtask` command runs every demo
  on desktop for a few frames in strict mode, where a logged error fails the run, so the demos are the engine's smoke
  tests ([Error handling](core/errors.md)).
- **Pure logic is checked by unit tests instead** (the math types, `WorldPos`, chunked reductions, reflection
  round-trips), since there is no engine in tests ([Lifecycle and memory](core/lifecycle.md)).
- **Demos grow into each other:** the window demo becomes the frame demo, the world becomes cubes, the cubes load a
  model, and from step 8 on the first room is the shared test case.

## 1. Workspace

- The cargo workspace at version 0.7.0, MPL-2.0, Rust 1.100 or later pinned, strict workspace lints, the formatter
  and clippy configuration, `.gitignore`, and `cargo xtask` as the internal build tool ([Principles and
  constraints](goals/principles.md), [Building and shipping](editor/shipping.md)).
- `cargo xtask` commands to run a demo on desktop and to build and serve it for a browser.
- Logging and profiling scopes (puffin) from the first crate.

Demos:
1. `hello` writes log lines at each level inside a profiler scope: to the terminal and the log file on desktop, to
   the browser console on the web.

Done when an empty engine crate builds for desktop and `wasm32-unknown-unknown`, clippy is clean, and `hello` runs on
both.

## 2. Start-up, windows, shut-down

- Start-up as one `async` function, a static per service, the winit event loop; on the web, a canvas and the loader
  library's start function ([Lifecycle and memory](core/lifecycle.md), [Building and shipping](editor/shipping.md)).
- **Windows, not a window, from the start,** proposed: the main window from the configuration, `window::main()` and
  `window::create`, the `Window` handle, close requests per window (closing the main window quits, closing another
  doesn't), and a surface per window following `resumed` and `suspended`. On the web the canvas is the only window
  and a second `create` returns an error ([Windowing and display](systems/windowing.md)). Why: one window or several
  changes how events are routed, how surfaces are held and how a frame renders, so single-window code isn't written
  first.
- **The GPU device, and each window's surface cleared to a colour,** proposed: the device with every available
  feature, checked at runtime. It is the first piece of renderer step 0, pulled forward so there are pixels and an
  overlay (step 3) early; the renderer's feature order is unchanged ([Renderer](rendering/README.md)).
- Shut-down as flush and exit, quit requested and quitting, `panic = "abort"` with a panic hook, `fatal_error`, and
  the crash message ([Error handling](core/errors.md)). Strict mode for the smoke runs, proposed.
- Headless as an additive build feature. **Fixed:** from the start ([Lifecycle and memory](core/lifecycle.md)).
- The typed configuration: sections registered in the entry point, the layered files, the command line, and the
  engine's early sections ([Configuration](core/configuration.md)).

Demos:
1. A window opens, cleared to a colour from the demo's own configuration section, and closes cleanly; the log shows
   the start-up order and the flush.
2. The window's title and size come from `config.toml`, and the command line (on the web, the loader's arguments)
   overrides them.
3. On desktop, more windows, each cleared to its own colour: closing one leaves the others, closing the main window
   quits. On the web, the second window's error is logged.
4. A demo setting causes a panic or a `fatal_error`: the crash message box (on the web, the page message) and the
   log, with the right path taken for each.
5. The headless build starts, logs and quits.

Done when windows open and close cleanly on desktop and in a browser, configured from a file and the command line.

## 3. Time, frame and jobs

- The frame's phases, the fixed-update loop with the 0.25 s clamp, pause and time scale ([Time and the
  frame](core/frame.md)). Frame timing: **Fixed:** the raw measured delta first; snapping and presentation timestamps
  come later (steps 16 and Later).
- The job system: `Task<T>`, the main-thread executor (`run_async`) also driving desktop start-up, `compute` on rayon,
  budgeted work on the web ([Jobs and async](core/jobs.md)).
- `Name`, the conventions' math types, and `WorldPos` with the f32/f64 build switch ([Conventions and
  names](core/conventions.md), [Game size and precision](goals/game_size.md)).
- Frame pacing: simulation once per frame and every window rendered, vsync, acquiring each surface at the end of the
  frame, the frame limiter ([Time and the frame](core/frame.md)).
- Memory attribution by logical counters ([Lifecycle and memory](core/lifecycle.md)).
- The development overlay with the puffin flamegraph and stats, the development console over settings, and pause,
  step and time scale from it ([Profiling and debugging tools](editor/debug_tools.md),
  [Configuration](core/configuration.md)). The overlay takes raw keyboard and mouse events from the window, since
  input comes in step 7.

Demos:
1. Each window's colour pulses with `time()`; a log line each second shows the frame rate and the fixed steps (about
   60). Vsync and the frame cap, set from the command line, change the frame rate.
2. A file read (a fetch on the web) and a long `compute` job run while the colours keep pulsing without a hitch;
   their results are logged.
3. The overlay shows the compute job in the flamegraph, and the memory counters; the console changes the demo's
   colour setting live, pauses, steps and changes the time scale.

Done when the game loop runs at a steady pace with the overlay and console, on desktop and the web.

## 4. The world in Rust

- The object model on hecs: entities, components with UUIDs and explicit registration, systems with declared access,
  the scheduler (non-conflicting systems in parallel, conflicting ones in registration order), commands applied after
  each group, and the schedule view ([Object model](world/object_model.md), [Time and the frame](core/frame.md)).
- The transform hierarchy and its propagation pass; transform-less entities ([Object model](world/object_model.md)).
- Reflection: serde values plus a derived schema; decide the derive crate here ([Reflection](world/reflection.md)).
- **An entity list in the development overlay,** proposed: the hierarchy, and the selected entity's components shown
  and edited through reflection. Why: it is the only view of the world before the renderer exists, and a first
  version of the editor's inspector and of runtime inspection ([Profiling and debugging
  tools](editor/debug_tools.md)).

Demos:
1. Several systems move a hierarchy of entities, and the demo logs a hash of every world transform after a set number
   of frames: the same hash with one worker thread, with many, and in a browser.
2. The schedule view shows which systems ran in parallel.
3. The overlay lists the entities; editing a component's field changes how its entity moves.

Done when Rust systems move a hierarchy of entities in parallel with the same result as on one thread, and a
component round-trips through its reflected form.

## 5. Renderer step 0: the foundation

- **Fixed:** the device with every available feature, checked at runtime (built in step 2); the render graph with
  injection points; views; the retained GPU scene; shared mesh buffers; `weshc` with shading-model wrapping and pass
  variants; debug labels; the `rgba16float` target with tonemapper None and sRGB output; compute passes and compute
  shaders from the start. Unlit on top ([Renderer](rendering/README.md), [Renderer
  architecture](rendering/architecture.md), [Shaders and materials](rendering/shaders.md)).
- The camera component, naming a window as its target ([Input and camera](systems/input_camera.md)).
- Owed at this step: an explanation of reversed Z, the depth convention ([Conventions and
  names](core/conventions.md)).
- **Debug drawing from Rust,** proposed here: lines, arrows, boxes and spheres in the world, for one frame or a
  duration, depth-tested or on top, drawn by an engine pass at an injection point, a no-op in release builds, plus the
  toggle for the bounds view ([Profiling and debugging tools](editor/debug_tools.md)). Why: the first engine pass at
  an injection point, and it makes step 8's physics visible. Text in the world comes with text (step 13), the script
  API with scripting (step 10).

Demos:
1. The render graph clears the `rgba16float` target and outputs sRGB; its labelled passes show in a GPU capture.
2. `weshc` compiles the Unlit shading model and a shader on it; a broken shader reports its file and line.
3. Step 4's hierarchy drawn as unlit cubes through a camera; on desktop, a second window shows it through a second
   camera.
4. A compute pass writes a texture that an unlit material samples.
5. Debug drawing: each cube's axes as arrows and a timed trail of spheres, and the bounds view toggled from the
   console.

Done when Rust code draws meshes with unlit materials through a camera, on desktop and the web.

## 6. Assets, project files and the CLI tools

- The project format: JSON files, one settings file per asset, and the regenerable import cache
  ([Assets and project files](world/assets.md)).
- One format library for the tools and the player: `weimport`, `weassetconv` and `weshc` on it.
- `Asset<T>` and `AssetRef<T>`, loading as a task, placeholders, freeing at frame end. The hot-reload API shape goes
  in now; reloading itself comes later. **Fixed:** "API first" ([Assets and project files](world/assets.md)).
- Packs and the manifest, loaded on desktop and the web, with texture formats chosen per target at build time and
  variants in the web build ([Building and shipping](editor/shipping.md)).

Demos:
1. The CLI tools turn a glTF and a PNG into the import cache and then a pack; the pack tool lists its contents.
2. The cubes demo loads a textured model from the pack; a demo setting delays the load, so the placeholder shows
   first.

Done when the CLI tools turn a glTF and a PNG into a pack that a Rust game loads and draws in a browser.

## 7. Input and windowing

- Input actions in one input asset, the per-tick input record, action sets and rebinding, keys by physical position,
  gamepads through gilrs, and the mouse ([Input and camera](systems/input_camera.md)).
- The windowing a single-window game needs on top of step 2: cursor modes, fullscreen, and application focus with
  held keys released ([Windowing and display](systems/windowing.md)). IME and the clipboard come with text fields
  (step 13), file drop with the editor (step 15).

Demos:
1. The input asset's actions are logged as they fire, from keyboard, mouse and gamepad, including one rebound at
   runtime.
2. A first-person camera walks around step 6's model with a locked cursor; holding a key and switching away from the
   game releases it.

Done when a first-person camera walks around a model loaded from a pack with keyboard and mouse or a gamepad, on
desktop and the web.

## 8. Milestone: the first room

A minimal first-person game in one level: a few rooms or hallways, something to find, a door it opens, and an exit.
Made in Rust with the CLI tools; no scenes, scripts, look, audio or UI yet. Proposed. Why: the engine is used by a game
long before the first game, and the steps after it have a shared test case. It is a tech demo, not the first game, so
it doesn't count towards 1.0 ([Principles and constraints](goals/principles.md#version-0-stays-flexible)).

- The physics it needs, pulled forward from step 12 behind the engine's own API: static colliders, a ray query, and
  the kinematic character controller, with the colliders debug view ([Physics](systems/physics.md)).
- Everything else is game code on steps 1 to 7: the level is a glTF in a pack spawned by Rust code, picking up is a
  ray query and a key press, a door opens by moving its transform, the exit is a distance check. Without in-game text,
  the ending resets the room or turns the screen black.
- Left out: scenes (step 9), scripting (step 10), the PSX look (step 11), audio (step 12), UI (step 13).

Demos:
1. Walking the level with collision: the character controller against static colliders, with the colliders view and
   the pick-up ray drawn.
2. Looking at the key and picking it up opens the door.
3. Reaching the exit ends the game.

Done when the room plays from start to exit on desktop and the web.

## 9. Scenes

- The scene asset, instances with scene properties, one root per scene, `load`, `unload` and `switch`, the start
  scene, global entities, and finding by type and tag ([Scenes](world/scenes.md)).
- **Writing entities to a scene file from Rust,** proposed: the serializer the editor will save with ([Assets and
  project files](world/assets.md#references-in-serialized-data)). Why: there is no editor yet to make scene files
  with.

Demos:
1. The first room's spawn code writes the level to a scene file once; the room then loads from that scene in a pack
   and plays the same.
2. The exit switches to a second room without a frame hitch: global entities survive, and entity and asset counts
   return to their level ([Lifecycle and memory](core/lifecycle.md)).
3. One door scene, instanced several times with different scene properties.

Done when a game made entirely in Rust switches between two scenes loaded from packs, without a frame hitch.

## 10. Scripting

- boa with TypeScript compiled by `tsc`, script components with exported properties, the callbacks, async waits and
  the settle point ([Scripting](scripting/README.md), [Script API](scripting/api.md)).
- Signals ([Signals](scripting/signals.md)).
- Script errors that never crash the engine, the loop limit, and the generated `.d.ts` ([Error
  handling](core/errors.md), [Documentation](editor/documentation.md)).

Demos:
1. A spinning script with an exported speed, changed in the overlay.
2. A "key picked up" signal, and the door opening after an `await`.
3. A demo setting loads broken scripts: the error is logged with its `.ts` line while the game keeps running, and an
   endless loop becomes a script error.

Done when the first room's game logic (step 8) is TypeScript scripts, with the same behaviour on desktop and the web.

## 11. Renderer steps 1 and 2: unlit complete, 2D basics, the first-game kit

- **Fixed (step 1):** alpha clip and blending with a sorted transparent pass, vertex colours, a sky, sprites with an
  explicit 2D sort order, pixel-art texture settings, and the compute API outside the frame.
- **Fixed (step 2):** fullscreen post materials at injection points, a low-resolution view upscaled with nearest
  filtering, distance fog, custom interpolators, and vertex-stage access to the clip-space position
  ([Renderer](rendering/README.md)).

Demos:
1. Renderer step 1: a small 2D scene of pixel-art sprites in sort order; glass and a sky in the first room; a compute
   job outside the frame logs its read-back result.
2. Renderer step 2: the first room with the PSX look.

Done when the first room has the PSX look as a user shading model plus post materials, and a small 2D scene works
too.

## 12. Physics and audio

- The rest of physics on rapier, after step 8's static colliders, ray query and character controller: rigid bodies,
  the other queries, collision layers, physics materials and surface types, and the four callbacks
  ([Physics](systems/physics.md)).
- The engine's own mixer (desktop output thread, web audio worklet), `Sound` assets, sources and one-shots with
  playbacks, the bus tree, the listener, and the web's audio unlock ([Audio](systems/audio.md)).

Demos:
1. A sound test: one-shots, a looping source, bus volumes from the console, and the web's audio unlock.
2. Positional sound in the first room: the door, a humming light.
3. Boxes to push, a trigger volume at the exit instead of the distance check, and footsteps by surface type.

Done when the first room uses trigger volumes and dynamic bodies, and has positional sound (the door, footsteps by
surface type), in a browser.

## 13. In-game UI and text

- UI screens of UI entities with CSS-shaped layout, UI units, input through the UI action set, full Unicode text with
  fonts and fallback, the localizable text type, and accessibility semantics ([In-game UI and
  text](systems/ui_text.md)).
- IME and the clipboard, for text fields ([Windowing and display](systems/windowing.md)).

Demos:
1. The first room's ending shows text instead of the black screen.
2. A main menu and a pause menu with mouse, keyboard and gamepad, and a text field with IME and paste.
3. Switching the language at runtime changes every text on screen.

Done when the first room has a main menu, a pause menu and in-game text that work with mouse, keyboard and gamepad.

## 14. Milestone: the first game

The small 3D project with a PSX look ([Why the engine exists](goals/why.md)), made in code and scripts with the CLI
tools, before the editor exists. Saves come in now if the game needs them ([Save games and
storage](world/save_storage.md)).

Done when the game is finished and playable on desktop and the web.

## 15. The editor

- The editor as a WutEngine game on egui: projects, the asset browser and import, the scene view, and the inspector
  built from reflection ([Editor](editor/README.md)).
- Every edit a change record through one edit API, undo and redo per document, atomic save, copy and paste.
- Play mode in its own process over the editor link, with live editing. **Fixed:** the play-mode window starts as a
  separate window; embedding it in the editor comes later.
- Building and shipping from the editor: desktop builds and the web export with its templates ([Building and
  shipping](editor/shipping.md)).
- Editor extensions in TypeScript and Rust; packages ([Packages](world/packages.md)).
- File drop, for dragging assets in ([Windowing and display](systems/windowing.md)).

Demos:
1. The editor opens the first room's project, imports a dropped file, and shows a scene in the scene view with a
   read-only inspector.
2. Moving the door, undoing, redoing and saving: the saved file's diff shows only that field. Copy and paste between
   scenes.
3. Play runs the room in its own process: logs, errors and profiler frames reach the editor, a live edit moves the
   door in the running game, and a crash in the game leaves the editor running.
4. Desktop and web builds from the editor.
5. A TypeScript inspector attribute, a Rust extension window, and a package.

Done when the first game's scenes can be opened, edited, played and built from the editor.

## 16. Jam-ready

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

Demos: each item above checked on its own, then a mini jam, proposed: a second small game made with the editor in a
weekend, before the test with someone new.

Done when someone who has never seen the engine makes a small game with it in an afternoon, and it runs in a browser.

## 17. After jam-ready

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

- **The first game before or after the editor (step 14).** Before means sooner, and the engine's Rust-only path gets
  tested. After means the first game also tests the editor.
- **Scenes (step 9) after the first room** is proposed so the room's own Rust code writes the first scene files.
  Before it, the room would start as a scene, but its file would be written by hand.
- **Scripting before or after the renderer's steps 1 and 2.** It's proposed earlier because the script API shapes
  how everything after it is used.
- **UI (step 13)** is the decision the design is least sure about. Building only what the first game needs, then
  revisiting the design, may be better than building it all at once.
- **Shadows and simple lighting** might be wanted before the first game, if the PSX look turns out to need them.
