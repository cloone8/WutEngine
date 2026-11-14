# Lifecycle and memory

How the engine starts and stops, where its services live, which lifecycle events games see, and the rules every
subsystem follows for memory. The concrete order of subsystems isn't designed here: it follows from their
dependencies when they are built.

## Start-up

- **Start-up is one `async` function on every target.** The steps read top to bottom, awaiting where they wait (the
  GPU, fetches); that function is the documented start-up order. Configuration's steps come first
  ([Configuration](configuration.md)), then logging, then the window and GPU. Until start-up finishes, the game runs
  nothing but its entry point's registrations.
  Why: on the web, getting a GPU is a promise and the main thread may never block; one function avoids a second
  start-up path that drifts apart and is tested least.
- **The driver differs per target.** On desktop, the job system's main-thread executor in blocking mode drives it to
  completion, parking the main thread until a waker fires; on the web, start-up is that executor's first task
  ([Jobs and async](jobs.md)). Why: one executor lets start-up await asset loads on every target.
- **Consequence of the blocking driver:** while it runs the event loop doesn't, so on desktop start-up can't await
  anything the event loop delivers (it runs inside winit's `resumed` and creates the window there directly), and
  nothing is drawn until it finishes. An animated splash during a long start-up would need the event-loop driver on
  desktop too; switching drivers doesn't change the start-up function.

## Where the engine's services live

- **A static per service.** Plain functions without a context argument (`time()`, `delta()`, reading a configuration
  section by type) reach their state through it; one game world per process (play mode is its own process).
  Why: the simplest way to back plain functions; an owned `Engine` value in a global slot would add little.
- **Conventions that follow:** services are initialized only by the start-up function, in its order; a service that
  must be replaceable while running holds a swappable value in its static; touching a service before it's initialized
  is an engine bug (an assert); there is no engine in tests, only pure-logic tests.

## Shut-down: flush and exit

- **Flush and exit in every build.** Only what must survive is done, then the process exits; threads aren't joined,
  and GPU objects and memory are left to the operating system. The same path serves a normal quit and `fatal_error`
  ([Error handling](errors.md)), which must work at any point of start-up, even before logging or the window exists.
  Why: teardown order is the classic source of crash-on-exit bugs, it only runs on desktop anyway, and statics never
  drop, so a teardown would be hand-written. Leaks are found while running instead (below).
- **Nothing durable may depend on shut-down running.** It runs on desktop when the game quits or calls `fatal_error`;
  not after a crash or a kill; on mobile a suspended app can be killed without another event; on the web closing a
  tab gives no reliable callback. Durable state is written at save points (lifecycle events below).
- **The flush path on a normal quit:**
  1. the game's quitting hooks: the "quitting" event, where a game saves, writes its own files or says goodbye to a
     server; registered by Rust code or received by scripts ([Signals](../scripting/signals.md)); run on the main
     thread with the engine fully working;
  2. the engine's flush steps: wait for pending file writes (including those the hooks just started), the last
     message over the editor link, any flush step a subsystem registered (such as a network disconnect message), then
     the log, flushed last so it records everything before;
  3. exit.
- **On a fatal error, a separate opt-in "on fatal error" hook** gets the message; the quitting hooks don't run. Its
  contract: the engine is degraded, so file I/O only (no rendering, no waiting on the GPU, no new loads), kept short. A
  timeout, enforced from another thread, exits the process anyway; a `fatal_error` raised inside the hook skips the
  rest of the hooks and goes straight to the flush steps. Games use it for a last save.
  Why: no hook written for a normal quit meets a degraded engine by accident.
- **A panic runs no hooks:** the panic hook only logs. Why: the game's state may be broken, so nothing is saved from it.
- **Player settings are written at the end of the frame they changed in** ([Configuration](configuration.md)), so the
  flush step's wait covers a setting changed just before quitting, and nothing depends on the quitting hooks.

## The window and the GPU can come and go

- **Only the renderer holds wgpu objects.** Everything else (game code, other subsystems, the editor) holds engine
  objects: assets, views, render targets, GPU jobs. Raw wgpu objects are available inside a render-graph pass or GPU
  job while it executes, not kept across frames. Rust programmers can still reach the device, documented as "may be
  replaced". Why: headless builds and clean ownership, and it keeps a renderer restart possible without planning it.
- **Surfaces follow `resumed` and `suspended`** on every platform: created on `resumed`, dropped before `suspended`
  returns, the held swapchain image included. Why: Android drops the surface on every suspend.
- **Device loss stays fatal** ([Error handling](errors.md)). Restarting the renderer in process is not planned; it is
  revisited when the web build shows how often losses happen there.

## Headless

- **Headless is a build feature, from the start.** The feature is additive: a default-on feature brings in the
  window, graphics, audio and input devices, and a headless build turns it off. Why: Cargo unifies features, so they
  must add, never remove; dedicated servers are already a separate build (the precision switch,
  [Game size and precision](../goals/game_size.md)).
- **Rendering and audio only read world data** in their own phases; headless skips those phases. Setting a material or
  playing a sound still changes data and is never an error; assets load without their GPU or audio upload step.
- **User-facing types live apart from their backends:** only the renderer and audio crates depend on wgpu and the
  audio backend, so game code compiles the same in both builds.
- **The frame loop runs without winit** as a plain timer-driven loop. Why: winit needs a display server on Linux,
  which a server doesn't have.
- **Without a window,** `fatal_error` and the panic hook write to stderr and the log; input on a server comes only as
  per-tick data ([Multiplayer](../goals/multiplayer.md)). The headless build is checked occasionally with clippy
  without default features.
- Kept possible: choosing headless at runtime in a graphics build.

## Lifecycle events games see

- **Started:** start-up finished, before the first frame; the place for the game's own Rust set-up after its
  registrations.
- **Quit requested** (window close, Cmd+Q, Alt+F4): cancellable, so a game can ask "quit without saving?"; the default
  accepts. Desktop only: mobile has no such event, and the web only the browser's generic dialog. Android's back button
  is an input event ([Input and camera](../systems/input_camera.md)).
- **Quitting:** quit is going ahead, not cancellable; runs the quitting hooks. Guaranteed on desktop for a normal quit;
  not after a crash or kill, and not on mobile or the web. The game's own `quit()` raises it too (but not "quit
  requested", since the game already decided). `quit()` on iOS logs a warning and does nothing; on the web it stops
  the game and shows a "the game has ended" page message.
- **On fatal error:** the opt-in hook above.
- **Backgrounded / foregrounded, on mobile and the web only:** the platform may kill the game soon without another
  event, so this is **the save point** there. It fires where the platform still gives time to save (on Android from
  `onStop`, on iOS on entering the background). Not on desktop: a minimized game keeps running and isn't killed;
  desktop games save in "quitting" and pause on focus lost ([Windowing and display](../systems/windowing.md)).
  Why: one clear meaning, "you may be killed soon", rather than an event that also fires on desktop minimize.
- **A development command "simulate background / foreground"** fires those events in play mode, so desktop development
  exercises the mobile save path.
- **Play mode:** the editor's Stop raises "quitting" (never "quit requested"), then ends the play-mode process after a
  timeout.
- Focus events come with [Windowing and display](../systems/windowing.md); mobile memory warnings come with mobile.

## Memory

- **Public APIs use the global allocator** (plain `Vec`, `String`, `Box`); custom allocators (arenas, pools, frame
  allocators) are a tool inside subsystems, using std's own `Vec<T, A>` and `Box<T, A>`. Why: an allocator parameter
  in a public type spreads through every caller's signature for a gain only the subsystem sees, and only `Vec` and
  `Box` support it.
- **Memory is attributed by counters in every build:** each subsystem counts what it owns, in units a user
  understands (loaded assets and their bytes per type, entities, components, script heap, GPU resources). They feed the
  overlay, budgets and baseline checks: entity and asset counts must return to their level after a scene unload,
  checked at a scene `switch` after the scripts' garbage is collected ([Assets and project files](../world/assets.md)).
  Why: cheap enough to ship, and in Rust most leaks are logical (a forgotten reference, a growing cache), which these
  counters see. They replace leak reports at exit.
- **Allocator tagging as an optional build feature,** off by default and turned on when investigating memory: a
  tagging global allocator with a header per allocation counts bytes per subsystem tag. Every subsystem enters its tag
  where its code runs (its phase, its systems, the script step, its jobs), the job system carries the tag of the code
  that queued a job into the job, a free is charged to the tag that allocated it, counters are kept per thread, and
  untagged memory shows as "untracked". With the feature off, entering a tag compiles to nothing.
  Why: it sees all memory, crates included, but its header costs (roughly 10% per allocation and 8–16 bytes each)
  suit an investigation, not every build. An allocator per subsystem wouldn't work: crates allocate through the
  global allocator.
- **The web's memory only grows,** so loading must not hold large temporary copies; a web game's peak use is its
  footprint for the session. Rules for loading, good practice on every platform:
  - **packs stay outside wasm memory on the web:** a fetched pack is a JavaScript `ArrayBuffer` (or stays in the
    browser cache), and only one asset's bytes at a time are copied into wasm memory;
  - **pack data is runtime-ready:** textures in the GPU format with their mips, meshes in the vertex layout of the
    shared mesh buffers, every asset in the form the runtime uses, so loading is "decompress, upload, free";
  - **no CPU copy after upload by default:** GPU assets drop their bytes once uploaded; a copy is kept only where
    something needs it (collision meshes, an asset marked readable by scripts);
  - **a cap on bytes in flight:** loads run concurrently only up to a byte limit;
  - **large assets in pieces:** textures uploaded mip by mip, mesh buffers in ranges, each piece freed after upload.
- **CPU out of memory is fatal** (Rust aborts).
- Later, where profiling or their topic needs them: the choice of global allocator (one line to swap); frame, arena
  and pool allocators inside subsystems; budgets per subsystem or level and what happens near them (with streaming);
  mobile memory warnings; a maximum memory size per web build. Virtual-memory tricks aren't designed (unavailable on
  the web).
