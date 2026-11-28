# Lifecycle and memory

How the engine starts and stops, where its services live, which lifecycle events games see, and the rules every
subsystem follows for memory. The concrete order of subsystems isn't designed here: it follows from their
dependencies when they are built.

## Start-up

- **The engine provides the entry point: a `#[wutengine::main]` attribute** on the game's entry function, from a
  proc-macro crate. It generates what each target needs: on desktop a `main` that runs the event loop, then flushes
  and exits; on the web a `main` that hands the application to winit and returns normally; later, Android's
  `android_main`. The game's source is the same on every target; the attribute's arguments are a first draft.
  Why: winit's `run_app` blocks on desktop but returns at once on the web, so a start function the game calls can't
  have one honest signature (a `-> !` on the web would need a control-flow exception out of `main`). Android has no
  `main` at all, and the attribute is one place for what must run before any game code (the panic hook). Why a
  proc macro over `macro_rules!`: an attribute on a normal function, and the proc-macro crate is needed for derives
  later anyway.
- **Start-up is a synchronous part, the main window, then one `async` function, the same on every target:**
  1. **The synchronous part** runs in the start function, before the event loop exists: configuration's steps
     ([Configuration](configuration.md)), logging, the game's entry point (its registrations), and the report of
     unregistered sections. A headless build runs it too.
  2. **The main window** is created in winit's first `can_create_surfaces`, directly from the event loop winit hands
     that callback, together with the wgpu instance and the window's surface (all synchronous). On desktop and the
     web that callback comes once, right after the loop starts.
  3. **The async part** follows from there: the GPU adapter, requested as compatible with that surface, the device,
     the surface's configuration, and whatever start-up awaits later (asset loads). It never creates windows itself.
     Why the surface first: with several GPUs, an adapter chosen without it may not present to the window.

  Read top to bottom, the three are the documented start-up order. Until start-up finishes, the game runs nothing
  but its entry point's registrations.
  Why: on the web, getting a GPU is a promise and the main thread may never block, while everything before the
  window is synchronous (player settings are a file or `localStorage`, and the loader hands in the project
  configuration). One path on every target avoids a second start-up path that drifts apart and is tested least, and
  creating the main window inside the callback needs no way for plain functions to reach winit's event loop.
- **Nothing before the main window may await.** A later need to (an asynchronous player-settings store on the web,
  say) would move the main window into the async part, created like any other window and awaited through its
  `ready()` ([Windowing and display](../systems/windowing.md)).
- **The async part is the main-thread executor's first task,** on desktop and the web alike: polled in winit's
  `about_to_wait` and woken through the event loop ([Jobs and async](jobs.md)). A headless run has no event loop, so
  there the executor drives it to completion in a blocking mode, parking the main thread until a waker fires.
  Why: one executor lets start-up await asset loads on every target, and one driver for every run with a window
  leaves no desktop-only start-up path to drift apart.
- **Where it runs in winit's flow:** on desktop winit's `run_app` blocks until the loop exits; on the web it
  registers the application and returns at once, and the browser drives the loop from then on.
- **Consequence: the event loop keeps turning during start-up,** so the async part may await anything the event loop
  delivers, and an animated splash during a long start-up stays possible. Frames start only after Started.

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
  replaced", and `None` in a headless build. Why: headless builds and clean ownership, and it keeps a renderer restart
  possible without planning it.
- **Surfaces follow winit's `can_create_surfaces` and `destroy_surfaces`** on every platform: created in
  `can_create_surfaces`, dropped before `destroy_surfaces` returns, the held swapchain image included. Only Android
  destroys surfaces; desktop and the web get one `can_create_surfaces` and never a destroy. The two may come back to
  back and repeat: start-up runs only in the first `can_create_surfaces`, later ones only recreate surfaces.
  Why: Android drops the surface on every suspend. winit's `resumed` and `suspended` are the app lifecycle instead,
  and exist only on iOS, Android and the web (below).
- **Device loss stays fatal** ([Error handling](errors.md)). Restarting the renderer in process is not planned; it is
  revisited when the web build shows how often losses happen there.

## Headless

A run is headless in a headless build, or in a graphics build started headless from the command line. Both behave
the same; "headless" below means either.

- **Headless is a build feature, from the start.** The feature is additive: a default-on feature, `interactive`,
  brings in the window, graphics, audio and input devices, and a headless build turns it off. Why: Cargo unifies
  features, so they must add, never remove; dedicated servers are already a separate build (the precision switch,
  [Game size and precision](../goals/game_size.md)).
- **A graphics build can also run headless,** turned on by a command-line switch, like Unity's `-batchmode
  -nographics`, Unreal's `-nullrhi` and Godot's `--headless`. It runs without winit, a window, or a GPU or audio
  device, as a headless build does. The switch's exact form comes with the command-line layer
  ([Configuration](configuration.md)). Why: the same build then serves as a local test server, or runs in CI without
  a display.
- **Engine code asks one question: is this run headless?** (`is_headless()`, always true in a headless build), and
  never checks the build feature to decide behaviour. `#[cfg]` is only for code that names what a headless build
  leaves out: the winit driver and the renderer and audio crates. Why: checking the feature gives the wrong answer in
  a headless run of a graphics build, and with few `#[cfg]`s both builds compile the same code.
- **Rendering and audio output only read world data** in their own phases (render preparation and the render graph);
  headless skips that backend work, decided inside the renderer and audio rather than by skipping phases. The game
  side of audio (each sound's time, `finished`) runs either way, at frame start ([Audio](../systems/audio.md)).
  Setting a material or playing a sound still changes data and is never an error; assets load without their GPU or
  audio upload step.
- **A headless build keeps the graphics and audio APIs and drops only their backends,** as Unity's dedicated server (a
  null graphics device) and Unreal's (NullRHI) do. Only the renderer and audio crates use wgpu and the audio backend,
  and the headless build leaves those crates out; user-facing types live outside them, so game code compiles the same
  in both builds.
- **The `wutengine` crate re-exports wgpu in both builds,** with no wgpu backend in a headless build, so custom render
  passes, GPU jobs and code that reaches the device compile unchanged. When headless, registered passes and GPU jobs
  never run and the device is `None`. Why: raw wgpu inside passes and GPU jobs is part of the API for Rust
  programmers, and wgpu without backends needs no system libraries or display server; the cost is compiling wgpu's
  core and naga in server builds.
- **Leaving code out of a headless build is the game's choice,** through its own Cargo feature that turns on the
  engine's; the engine never requires it. Like `UNITY_SERVER` in Unity and `UE_SERVER` in Unreal.
- **The frame loop runs without winit** as a plain timer-driven loop. Why: winit needs a display server on Linux,
  which a server doesn't have.
  - **At a fixed tick rate,** 60 per second until configuration makes it a setting (common server rates are 20 to
    128).
  - **Deadline-paced:** each deadline is the previous one plus the interval, so sleep overshoot doesn't add up. A loop
    more than one interval late re-anchors to now instead of running a burst of frames; the game delta's clamp
    already limits catch-up fixed steps.
  - **std's `thread::sleep` is the timer:** precise on Linux, and on Windows from 10 version 1803 on (a
    high-resolution waitable timer). Not `park_timeout`, which on Windows waits in whole timer ticks (about 15.6 ms).
- **Without a window,** `fatal_error` and the panic hook write to stderr and the log; input on a server comes only as
  per-step data ([Multiplayer](../goals/multiplayer.md)). The headless build is checked with clippy without default
  features.
- Kept possible: server packs without GPU and audio data, stripped when packing (as Unity's dedicated-server build
  and Unreal's server cook do), since assets already load without their upload step.

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
  event, so this is **the save point** there. It fires where the platform still gives time to save:
  - on Android from `onStop`, which is winit's `suspended` there;
  - on iOS on entering the background. winit's `suspended` there is resigning active, which is earlier and also fires
    for an incoming call or Control Center, so this needs its own hook;
  - on the web when the page is hidden, from the engine's own `visibilitychange` listener. winit's `suspended` there
    only means the page went into the back/forward cache, and its `Occluded` also fires when the canvas scrolls out of
    view.

  Not on desktop: a minimized game keeps running and isn't killed;
  desktop games save in "quitting" and pause on focus lost ([Windowing and display](../systems/windowing.md)).
  Why: one clear meaning, "you may be killed soon", rather than an event that also fires on desktop minimize.
- **Rust hooks can be registered at any time, from any thread;** they run on the main thread, outside a job, in
  registration order. A hook registered while that event's hooks run fires from the next time on, and a Started
  hook registered after start-up never runs. Why: registering early is the common case, but nothing gains from
  forbidding a later one, and per-event lists that are taken out while they run make it safe.
- **Quit requested runs every handler, and any cancel wins:** a cancel can't be undone, so the outcome doesn't depend
  on the order, and a later handler can see an earlier cancel and skip its own dialog. Handlers return at once:
  cancel now, ask the player in later frames, then call `quit()`. Once a quit is scheduled, quit requested no longer
  fires.
- **`quit()` schedules the quit and returns;** the driver carries it out at its next safe point on the main thread,
  then runs the quitting hooks. A scheduled quit is final: later calls are ignored, and a `quit()` inside a quit
  requested handler overrides a cancel. Why: the quitting hooks need the engine fully working on the main thread,
  and `quit()` may be called from any thread.
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
