# Open questions

What the design hasn't settled yet, grouped by chapter. Three kinds:
- **Decide:** a choice still to make.
- **Implementation:** left to implementation on purpose; decide while building it.
- **Verify:** an assumption the design rests on that hasn't been checked or measured yet.

When an item is settled, its answer goes into the chapter and the item leaves this page. Items marked with a
planning step, such as **(step 4)**, are settled in that step; [Planning](planning.md) lists them there too.

## Goals

- **Decide:** whether runtime entity IDs are allocated in the same order on every run. It matters only for replays
  and lockstep networking ([Multiplayer](goals/multiplayer.md)).

## Engine core

- **Decide:** the script API's default angle unit, probably degrees ([Conventions and names](core/conventions.md),
  [Script API](scripting/api.md)).
- **Implementation:** for names ([Conventions and names](core/conventions.md)):
  - the hash function;
  - the name table's implementation (locking, sharding);
  - whether release builds keep the name strings;
  - whether bone names are resolved to indices at load.
- **Implementation:** the concrete subsystem order at start-up and shut-down, the fatal-error hook's timeout, and a
  splash window during start-up ([Lifecycle and memory](core/lifecycle.md)).
- **Decide (with the command line):** the switch that starts a graphics build headless, and whether it is a
  configuration value or read before the configuration ([Lifecycle and memory](core/lifecycle.md#headless),
  [Configuration](core/configuration.md)).
- **Decide:** for quitting on winit 0.31 ([Lifecycle and memory](core/lifecycle.md)):
  - how `quit()` stops the game on the web. It throws today, which from inside a winit callback would unwind through
    winit's web runner mid-dispatch. To start: quit through winit's `exit()`, then show the page message.
- **Decide:** termination signals on desktop, for interactive and headless runs alike: `SIGINT` and `SIGTERM` on Linux
  and macOS; Ctrl+C, Ctrl+Break and closing the console on Windows. Today they end the process without the log flush
  or the quitting hooks, which matters most for a dedicated server stopped by a service manager
  ([Lifecycle and memory](core/lifecycle.md#lifecycle-events-games-see)). To decide:
  - which event a signal raises: "quit requested" (cancellable) or straight to "quitting";
  - what a second signal does while the first is being handled (to start: exit at once, after a log flush);
  - how the handler reaches the loop: it may only set a flag (and on Windows it runs on a thread of its own), so the
    winit driver needs a wake-up through its event-loop proxy, and the headless driver a check each tick;
  - the time limits: a service manager kills the process after its stop timeout, and Windows ends a process a few
    seconds after its console closes;
  - an existing crate (`ctrlc`, `signal-hook`) or the engine's own handlers.

  To start: the first signal raises "quitting", like a normal quit; a second exits at once after a flush.
- **Implementation:** for errors ([Error handling](core/errors.md)):
  - the library that shows the desktop crash message box;
  - the script loop-limit default (10 million iterations per call to start, then tuned);
  - on the web, making sure the player sees the out-of-memory message rather than an earlier validation error.
- **Decide:** how frame-time snapping detects a fixed refresh rate. To start: winit's reported rate confirmed by the
  measured average, re-estimated on monitor, vsync or fullscreen changes. Revisit when snapping is built
  ([Time and the frame](core/frame.md)).
- **Decide:** the latency setting (two frames in flight by default, a "low latency" option with one). Also watch the
  60 Hz fixed rate during implementation ([Time and the frame](core/frame.md)).
- **Implementation:** for the job system ([Jobs and async](core/jobs.md)):
  - the executor, pool layout and thread counts, affinity and the I/O thread count;
  - the main-thread time slice (length and split), the batch cut-off value and the queue design;
  - names such as `run_async`, `compute`, `jobs::for_each`, `map_reduce`, `join`, `scope` and `try_take`, which are
    first drafts.
- **Decide:** for configuration ([Configuration](core/configuration.md)):
  - whether play mode gets its own override file (to start, it reads the development file);
  - whether more override axes are needed;
  - how typed field handles into nested structs compose;
  - the module name (`settings::` or `config::`).
- **Verify:** for engine core:
  - frame pacing on Windows and Linux (X11 and Wayland), including whether a frame cap paces better there;
  - whether the web page gets any say in vsync;
  - the allocator tagging cost on Linux, Windows and wasm;
  - whether boa's loop-limit error can name the script line;
  - whether winit reports the OS ending the session on desktop (Windows doesn't);
  - mobile backgrounding beyond winit's events, and iOS `quit()` behaviour;
  - the cost of the plain distance loop over collider-less entities at jam-to-indie counts.
- **Decide (step 3):** reserved names for the engine's configuration sections, with a registration error that names
  a clash between a game's section and the engine's ([Configuration](core/configuration.md)).
- **Decide (step 3):** separate default log levels for the engine with its dependencies (wgpu, naga, ...) and for
  game code, also as the editor console's default filters ([Profiling and debugging
  tools](editor/debug_tools.md)).
- **Decide (step 3):** what `Color` means in code: linear in memory; hex and byte constructors taking sRGB as CSS
  does, next to explicit linear ones; alpha defaulting to 1 ([Conventions and names](core/conventions.md)).
- **Decide (step 3):** an already-finished task (`Task::from_value`), so a cache hit and a load share one code path
  ([Jobs and async](core/jobs.md)).
- **Decide (step 4):** the order of commands and new entity IDs coming from parallel systems: for example buffers
  per system merged in registration order, then chunk order, then call order; whether a Rust `spawn` returns a
  usable entity ID at once; and that IDs which feed any ordering (sorts, tie-breaks) are minted at deterministic
  points. Related to the entity ID question under Goals ([Object model](world/object_model.md)).
- **Decide (step 4):** the scheduler's placement rule: a system goes after the last set that holds any earlier
  system it conflicts with, not into the first set without a conflict, which breaks registration order
  ([Time and the frame](core/frame.md)).
- **Decide (step 4):** how a system is referred to in before/after orderings (typed handles), a cycle check and an
  error for orderings across phases at registration, and whether systems are registered only in the entry point
  ([Time and the frame](core/frame.md)).
- **Decide (step 4):** one general rule for engine services called from parallel systems: read-only for the phase,
  or queued and applied in registration order, then call order, as audio and signals already do
  ([Time and the frame](core/frame.md)).

## The game world

- **Decide:** which derive produces the reflection schema (schemars, facet, or the engine's own), when reflection is
  built ([Reflection](world/reflection.md)).
- **Decide:** for assets ([Assets and project files](world/assets.md)):
  - whether the editor link identifies entities by UUID or by a per-session mapping;
  - compact indices instead of UUIDs in build packs;
  - whether lazy conversion in play mode works out (a goal, to try).
- **Implementation:** for assets ([Assets and project files](world/assets.md)):
  - memory budgets, the bytes-in-flight and upload caps;
  - the asset hash function and placeholder looks;
  - how play mode learns the build's contents;
  - the native texture upload path;
  - the main-thread executor re-polling ready futures within its time slice, so a load advances several steps per
    frame;
  - per-type load and upload code;
  - the `Asset<T>` name, a placeholder.
- **Decide:** for scenes ([Scenes](world/scenes.md)):
  - the API shape (`spawn` and friends), expected to settle while building it;
  - identity inside instances, to be researched more carefully when built;
  - apply/revert, and how scene properties show in the editor, designed with the editor.

  File layout, how a spawn copies a scene, override storage, the tag index and main-scene bookkeeping are left to
  implementation.
- **Implementation:** for packages ([Packages](world/packages.md)):
  - the package file's name and format, the folder name and the CLI command name;
  - how editor-only code is marked, designed with editor extensions.
- **Implementation:** for saves ([Save games and storage](world/save_storage.md)):
  - the header layout, file and backup naming, size and name limits, and folder-name rules;
  - the IndexedDB wrapper crate;
  - API names such as `saves.write`.
- **Verify:** that the native `tsc` resolves package imports through the generated `tsconfig.json` without Node, and
  with which `moduleResolution` mode ([Packages](world/packages.md)).
- **Decide (step 4):** local and world transforms as separate component types, the world one written only by
  propagation, so "world transforms as of the start of the phase" works with access tracked per type
  ([Object model](world/object_model.md)).
- **Decide (step 4):** removed components and despawned entities as part of change tracking, read by every
  subsystem with proxy components (physics, audio, renderer) ([Object model](world/object_model.md)).
- **Decide (step 6):** for assets ([Assets and project files](world/assets.md), [Building and
  shipping](editor/shipping.md)):
  - where the IDs of several assets imported from one source live (for example in the source's settings file, keyed
    by a stable sub-path), and what a rename or removal inside the source does;
  - the asset type's UUID in every pack and manifest entry, and a format version per cache entry and pack header;
  - formats spanning several files: readers get a resolver for related files, imports record the files they read,
    and staleness covers sources, settings and the importer version;
  - optional asset fields as `Option<Asset<T>>`, shown as "None" in the inspector;
  - bulk data made in the editor (heightmaps, baked maps) written as standard source formats next to a JSON asset;
  - asset path rules: project-relative, `/`-separated, case-exact, with a build error on a case-only clash.

## Scripting

- **Decide:** for the script API ([Script API](scripting/api.md)):
  - the destroy callback's name and its order relative to children's;
  - the cap on continuations per drain;
  - the wait API names, which are first drafts;
  - the rule for enums with data, which the camera's field-of-view form refers to.
- **Decide:** the signal API's names, and how a `Signal<T>` field is reflected ([Signals](scripting/signals.md)).
- **Decide:** for visual scripting ([Visual scripting](scripting/visual_scripting.md)):
  - how the graph editor learns the types of user TypeScript functions: parsing `tsc --declaration` output, or
    the undocumented `tsc --api` protocol;
  - whether generated files live in the project or only in a build folder.
- **Verify:** for scripting ([Scripting](scripting/README.md)):
  - that the native TypeScript 7 `tsc` runs without Node;
  - the boa spike on Windows (MSVC), and in Chrome and Safari.
- **Later:** ask boa's maintainers about contributing debugger hooks upstream.

## Rendering

- **Decide:** for the render graph ([Renderer architecture](rendering/architecture.md)):
- **Decide:** which scene owns world-wide environment lighting, the way gravity is a project setting
  ([Scenes](world/scenes.md), [Renderer architecture](rendering/architecture.md)).
  - one command encoder per pass (as decided) or passes grouped into chunks with one encoder each (as the recording
    notes sketched); profile before splitting further;
  - how several custom passes at one injection point are ordered;
  - the exact shape of a render-graph pass;
  - how the readers of a camera cycle are found and the previous frame's image is kept.
- **Implementation:** for the renderer ([Renderer architecture](rendering/architecture.md)):
  - target formats beyond the main colour target;
  - vertex formats, attribute grouping and the position precision target;
  - the mesh simplification library, LOD error metric and hysteresis;
  - the sort key's bit layout and pre-exposure;
  - the grading LUT size;
  - how glTF's emissive strength maps to nits;
  - the compute buffer-layout derive, the engine's own for now.
- **Verify:** for shaders ([Shaders and materials](rendering/shaders.md)):
  - whether browsers accept WGSL's `pointer_composite_access`;
  - whether naga is already in the web build, and the size runtime shader generation adds if not;
  - whether WESL tools tolerate the engine's own attributes;
  - which platforms lack runtime shader compilation.
- **Decide:** whether texture variants get one manifest location per variant or one manifest per variant, and how
  assets are compressed inside packs ([Building and shipping](editor/shipping.md)).
- **Decide (step 5):** for shaders ([Shaders and materials](rendering/shaders.md)):
  - which capability set each variant is validated against (the WebGPU core for baseline variants), so the runtime
    never picks a variant that needs a capability the device lacks;
  - the variant cache key: the content of all imports (transitively), the `weshc` version, the capability target, the
    pass and binding axes;
  - a source map in `weshc` (keep line structure, map spans to file and line through imports and `@if`), with the
    graph's line-to-node table on top;
  - engine-owned bind groups (view, object) reflected from the engine's own WGSL with a test, and an actionable error
    when a user shader binds into them;
  - how a dropdown keyword with several values lowers to `@if`, errors for undeclared or out-of-range keywords, and a
    variant key normalised with defaults;
  - what a shading model gets when a mesh lacks a stream it reads (defaults fed by the wrapper, or a check at
    material assignment logged once, never per draw);
  - the pipeline cache on disk: keyed by adapter, driver and the game's identity, treated as untrusted input, saved
    after loads rather than on exit;
  - formatted debug labels compiled out of release builds.
- **Decide (step 5):** state that lives across frames per view (history buffers, auto exposure, query sets), owned
  outside each frame's graph and imported into it, freed with the view ([Renderer
  architecture](rendering/architecture.md)).

## Engine systems

- **Decide:** which monitor the main window opens on and how that is stored, what happens to a saved window size that
  doesn't fit, and how Cmd-Q on macOS can become a cancellable "quit requested" ([Windowing and
  display](systems/windowing.md)).
- **Verify:** for windowing ([Windowing and display](systems/windowing.md)):
  - that winit keeps a window's OS-scaled size when its scale factor changes, on every platform;
  - whether a web fullscreen request made the frame after a click keeps the click's user activation;
  - frame times with two and three vsync'd windows.
- **Verify:** for input ([Input and camera](systems/input_camera.md)):
  - on macOS, the mouse motion event rate with a high-rate mouse, and summed deltas for slow and fast moves with
    acceleration on and off (this opens a window: ask first);
  - Android's physical keys and back button on a real device;
  - what a locked cursor yields where raw motion doesn't exist.

  Decide: whether to send the Windows absolute-mode mouse fix to winit upstream.
- **Decide:** the localization message syntax (Fluent or Unicode MessageFormat 2), when localization is built. The
  in-game UI design as a whole is the decision the design is least sure about: adjust it readily once building the UI
  shows the need ([In-game UI and text](systems/ui_text.md)).
- **Implementation:** for UI ([In-game UI and text](systems/ui_text.md)):
  - hover and press states, stick repeat timing, drag thresholds and focus memory per screen;
  - glyph rasterization and atlases, batching and clipping;
  - component names.
- **Implementation:** for audio ([Audio](systems/audio.md)):
  - how the per-frame batch reaches the desktop output callback;
  - the stop fade length and the distance curve kinds;
  - finding which sources are in range;
  - where decoding runs, which decoder crates stand in, and the clip length above which "compressed in memory" is
    the default;
  - the volume slider's mapping to gain;
  - the rule for several listeners.
- **Verify:** for audio on the web and mobile ([Audio](systems/audio.md)):
  - the audio worklet in Chrome and Safari, with large clips, streamed sounds, many messages per frame and hidden
    tabs;
  - whether a gamepad press unlocks audio;
  - which formats Safari decodes;
  - iOS ringer and interruption behaviour.
- **Implementation:** for physics ([Physics](systems/physics.md)):
  - how a non-uniform scale bends spheres and capsules;
  - whether fixed colliders get their own query structure;
  - reporting events only where someone listens.
- **Verify:** for physics ([Physics](systems/physics.md)):
  - the speed cost of rapier's `enhanced-determinism`, needed only for the multiplayer door;
  - the character controller's autostep cost;
  - the query sync's cost on the engine side, on the web, for child colliders and for triangle meshes.
- **Decide (step 3):** standard cursor shapes per window (arrow, text, resize, grab, pointer), with egui driving
  the shape while it has the pointer ([Windowing and display](systems/windowing.md)).
- **Decide (step 5):** the camera viewport rectangle's origin: top-left, matching screen positions
  ([Input and camera](systems/input_camera.md)).
- **Decide (step 7):** for input ([Input and camera](systems/input_camera.md)):
  - a press threshold with hysteresis per binding, for an analog input bound to a button action;
  - how several devices driving one player combine on one action, and a last-used-device query for button prompts;
  - the axis directions of mouse motion and scroll;
  - whether a key without a mapping can still be bound (by raw code, not portable between machines).
- **Decide (step 8):** whether a `Collider` has a local centre and rotation, applied through its entity's world
  transform; and, when 2D physics comes, how the 3D transform maps to 2D (the XY plane, the angle about Z, what tilt
  and scale do) ([Physics](systems/physics.md)).
- **Decide (step 12):** a pause per audio source that keeps its position, or the documented workaround
  ([Audio](systems/audio.md)).

## Editor and tools

- **Decide:** for the editor ([Editor](editor/README.md)):
  - how writes from per-frame edit-mode callbacks count as edits, decided with those callbacks;
  - whether compiled-in Rust extensions can be switched off at runtime for a recovery mode;
  - the crash-recovery journal's layout and location, when it is built.

  The undo memory limit, grouping async edit-mode writes and the clipboard wrapper are left to implementation.
- **Decide:** the web loader library's exact shape, a first draft ([Building and shipping](editor/shipping.md)).
- **Verify:** whether the `windows-gnu` toolchain avoids installing the Visual Studio Build Tools ([Building and
  shipping](editor/shipping.md)).
- **Implementation:** how doc strings are compiled out of the release player ([Documentation](editor/documentation.md)).
- **Decide later:** how the versioned online documentation is served and deployed
  ([Documentation](editor/documentation.md)).
- **Decide (step 3):** for the development overlay ([Profiling and debugging tools](editor/debug_tools.md)):
  - a registry of overlay panels (Rust only, development builds only) with an open/close hook;
  - profiler scopes recording only while someone watches (the overlay's profiler open, or the editor link
    subscribed), with a setting to always record;
  - built-in diagnostic panels: GPU adapter, driver, features and limits (also logged at start-up), each window's
    surface format, present mode and latency, a browsable settings view; input devices with live action states come
    with step 7;
  - who gets input and the cursor while the overlay is open: consuming what egui wanted in the previous frame,
    unlocking the cursor while the overlay has focus, and restoring the game's mode afterwards;
  - the engine's egui integration: fed from ordered window events (positions, repeats, physical keys, pointer gone,
    scroll in pixels), never rebuilt from polled state; user textures (render textures shown in egui) from the start;
    drawn last, after tonemapping, at full resolution, blending in encoded values;
  - who draws the flamegraph: the engine's own view on egui, or upstream puffin_egui with its egui version lag.
- **Decide (step 6):** conventions for the CLI tools: data on stdout and logs on stderr, a non-zero exit when any
  input failed, and a JSON dump of a cache or pack entry ([Building and shipping](editor/shipping.md)).
- **Decide (step 14):** for shipped builds ([Building and shipping](editor/shipping.md)): the Windows executable's
  icon and version resource, the GUI subsystem attaching to a parent console for command-line use, a build identity
  in logs and crash messages (a git commit optional, never required); and a third-party licence notices file in every
  build, the web included.
- **Decide (step 15):** for the editor ([Editor](editor/README.md)): a project marker file with a real check of the
  editor against the project's locked engine version; opening or switching a project in a fresh editor process; how
  extensions place menu items.
- **Placeholder names,** first drafts like those listed above: `AudioSource`, `Sound` and `audio.play`
  ([Audio](systems/audio.md)); `RigidBody`, `Collider` and `physics.raycast` ([Physics](systems/physics.md)); `edit.set`
  and `edit.group` ([Editor](editor/README.md)); the term "UI screen" ([In-game UI and text](systems/ui_text.md)).
