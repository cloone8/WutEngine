# Open questions

What the design hasn't settled yet, grouped by chapter. Three kinds:
- **Decide:** a choice still to make.
- **Implementation:** left to implementation on purpose; decide while building it.
- **Verify:** an assumption the design rests on that hasn't been checked or measured yet.

When an item is settled, its answer goes into the chapter and the item leaves this page.

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
- **Implementation:** the concrete subsystem order at start-up and shut-down, the fatal-error hook's timeout, the
  headless feature's name (one feature or one per subsystem), and a splash window during start-up
  ([Lifecycle and memory](core/lifecycle.md)).
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
  - whether winit's web backend reports tab visibility as `Occluded`;
  - whether winit reports the OS ending the session on desktop (Windows doesn't);
  - mobile backgrounding beyond winit's events, and iOS `quit()` behaviour;
  - the cost of the plain distance loop over collider-less entities at jam-to-indie counts.

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
- **Placeholder names,** first drafts like those listed above: `AudioSource`, `Sound` and `audio.play`
  ([Audio](systems/audio.md)); `RigidBody`, `Collider` and `physics.raycast` ([Physics](systems/physics.md)); `edit.set`
  and `edit.group` ([Editor](editor/README.md)); the term "UI screen" ([In-game UI and text](systems/ui_text.md)).
