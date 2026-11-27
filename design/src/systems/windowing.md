# Windowing and display

Windows, closing and quitting, fullscreen, pixel units, the cursor, focus, text input, the clipboard, file drop, and
the window settings in the configuration. The engine decides only what game code can't; helpers are
[Later](../later.md).

## A window is an engine object created by a plain function

- **The main window** is created at start-up from the configuration, unless the run is headless
  ([Lifecycle and memory](../core/lifecycle.md#headless)). A game with one window never calls the window API.
- **`Window::main()` returns an `Option<Window>`:** `None` when headless, where the window API still compiles
  because game code compiles the same in both builds ([Lifecycle and memory](../core/lifecycle.md)), and `null` in
  scripts. On a client it is always `Some` from the Started event on: the main window is open by the time start-up
  finishes. On mobile the surface comes and goes; the handle stays. Why: an explicit absence instead of a window that
  silently does nothing on a server.
- **The window API is an error in the game's entry point,** which runs before the main window exists and only
  registers ([Lifecycle and memory](../core/lifecycle.md#start-up)); game set-up that needs windows goes in Started.
  Why: a `None` from `Window::main()` there would read as "headless", and a quietly queued window as a working one.
- **More windows:** `Window::create(settings)` (title, size, mode, resizable, ...) returns a `Window` handle at once;
  the engine creates the OS window in the event loop's next iteration. The handle has the window's state and methods
  (size, scale factor, focus as a property, close) and is the one runtime control of a window; what the window
  reports is the truth. Cameras name a `Window` as their target. Scripts get the same API. When headless, the handle
  starts out failed, its error saying the run is headless.
  - **An explicit state:** pending, open, failed (with its error) or closed. **`ready()`** returns a task that
    completes with the open window or the error ([Jobs and async](../core/jobs.md)): awaited in async code, checked
    each frame otherwise, a promise in scripts.
  - **Commands given before the window opens are queued** and applied in order once it does.
  - **Getters report the last known values,** never made-up ones: while pending, the requested settings (the state
    says they are provisional); after closing, the values it had.
  - Why a handle at once: code can keep it, give it to a camera or set its title right away, and synchronous frame
    code doesn't poll a task to get one. Why not a promise of synchronous creation: winit creates windows only inside
    its own callbacks, and a window's real state arrives later anyway (its size on Wayland once the compositor
    configures it, its surface on Android when the system allows one). It also keeps winit's event loop out of reach
    of plain functions.
- **The handle reads a snapshot, refreshed at frame start** (phase 0, [Time and the frame](../core/frame.md));
  setters are queued and applied on the main thread at the next frame start, as configuration sets are. Why: systems
  running in parallel read window state safely, and a value never changes in the middle of a phase.
- **`Window::create`'s settings default to a windowed window,** with an optional minimum and maximum size. Why: a
  fullscreen default forces every tool window to override it.
- **Where a platform has one window** (Android, the web, and iOS for now), creating a second fails: its state becomes
  failed and `ready()` completes with the error. On the web the window is the page's canvas, given by the page or
  appended. Kept possible: several canvases on one page,
  which winit already supports, so the one-window rule on the web is the engine's own.
- Why windows aren't entities: they are an engine service, not game objects, and aren't saved in scenes
  ([Principles and constraints](../goals/principles.md)). The editor's needs (the play-mode window, floating tools
  later) are met by the same function.

## Closing windows and quitting

- **The main window's close button is "quit requested":** cancellable, the default accepts, then "quitting"
  ([Lifecycle and memory](../core/lifecycle.md)).
- **Another window's close button** sends a close request for that window; by default the window closes. A game or
  tool that handles the request can keep it open.
- **Closing the main window quits; closing any other window never does.** Why: a tool window outliving the game window
  shouldn't keep the game alive, and closing a tool window shouldn't end it.

## Fullscreen: windowed and borderless

- **Modes: windowed and borderless fullscreen,** on a chosen monitor (the window's current one by default). On macOS
  borderless uses its own Space.
- **No exclusive mode (a display-mode change) for now.** Why: wgpu has no exclusive fullscreen on any backend, a
  display-mode change misbehaves on macOS and doesn't exist on Wayland or the web, and flip-model borderless on Windows
  is as efficient as exclusive. A lower resolution for speed is the render scale. The mode is a `#[non_exhaustive]`
  enum, so exclusive can be added if a platform needs it.
- **Web:** fullscreen needs a user gesture and the player can leave it at any time. The engine reports the actual mode
  after a change; a request outside a gesture fails with a logged warning.

## Pixels and units

- **What's drawn and pointed at is in the window's physical pixels:** cursor and touch positions, the camera's screen
  conversions for window targets, surface, render and render-texture sizes. One unit, the same as the image.
- **Window geometry is in OS-scaled units:** the configuration's windowed size, `Window::create`'s size, the handle's
  size (get and set) and monitor sizes. *OS-scaled units* are the unit the OS sizes windows in: points on macOS,
  device-independent pixels on Windows, CSS pixels on the web; physical pixels are OS-scaled units times the window's
  scale factor.
- **The two units are kept apart by type,** in Rust and in scripts ([Script API](../scripting/api.md)); a value of one
  can't be passed as the other. UI units are a third ([In-game UI and text](ui_text.md)).
- **The guarantee: a saved size restores the same window.** The size getter returns the type its setter takes, so a
  game saving `window.size()` can't grow the window each launch. The handle's size in physical pixels is the surface
  size, one value under one name.
- Why OS-scaled geometry: a creator's "1280×720" is the same apparent window on every screen; physical sizes open a
  small window on every high-DPI display. Cost: two units in the window API, kept apart by type. Known ceiling: a
  window of an exact pixel size at a fractional scale can't always be hit; a pixel-art game scales inside the window
  instead. Kept possible: a pixel-exact size request.
- **A scale-factor change** (another monitor, browser zoom) is meant to keep the OS-scaled size and change the pixel
  size; the engine handles it.
- **The OS scale factor is a property of the window,** used by whatever should follow the OS: the editor and egui,
  app-like games, or a game's optional "follow system scaling" setting. **Games don't scale their UI with the OS by
  default;** game UI scaling is in [In-game UI and text](ui_text.md).
- **Conversions work in the target's own units** ([Input and camera](input_camera.md)), and the cursor position lives
  in input, not on the `Window`, so a single-window game unwraps one `Option`, not two.
- **On the web** the configured size applies only to a canvas the engine appends; a canvas the page provides keeps the
  size the page gives it.
- **Window position** isn't part of window geometry yet. Kept possible, with the monitor choice.
- **Kept possible:** opting a window out of high-DPI (the OS upscales everything).

## Cursor

- **Three modes: free, confined to the window, locked** (hidden, motion only; mouse look reads raw motion while
  locked), plus **visible or hidden** in free and confined modes. Cursor mode and visibility are plain functions acting
  on the main window.
- **The engine reports the mode actually in effect.** Where a platform can't do a mode, it says so (a warning logged
  once) rather than guessing. Where lock is missing the engine may emulate it (confined, hidden, re-centred), and an
  emulated lock reports "locked". On the web, lock needs a user gesture and the player can leave it; the reported mode
  changes back.

## Focus and minimize

- **One event for games: the application gained or lost focus** (none of its windows is focused any more, or one is
  again), for pause menus. Per-window focus is a property on the `Window` only. Scripts get the event as a signal on
  an engine service object ([Signals](../scripting/signals.md)).
- **On application focus loss the engine releases every held key and button** (moving between the game's own windows
  doesn't count). Why: keys pressed during alt-tab would otherwise stay stuck, since their release goes to another
  application. The per-step record sees the releases.
- **No automatic pause or mute.** The game decides; a template can show a pause menu on focus loss. Why: pausing by
  default surprises networked games and background music.
- **A minimized window doesn't render; an unfocused game runs at a low frame cap** (a setting)
  ([Time and the frame](../core/frame.md)). Focus isn't "backgrounded", which is mobile and web only.

## Text input, IME, clipboard, file drop

The runtime features the editor needs, at their minimum:
- **IME is off by default** and turned on while a text field has focus, with the caret's area for the candidate
  window; the text field (in-game UI's or egui's) calls it. On mobile the same call shows the soft keyboard. Kept
  possible: IME on the web, which winit lacks.
- **Clipboard: text only to start.** Setting text is a synchronous plain function. **Pasting arrives in the focused
  text field as an input event,** like IME text; on the web that is the normal paste path, with no permission prompt.
  A general clipboard read by game code waits until something needs it; images later.
- **File drop:** an event with file paths on desktop (the editor's drag-to-import). Outgoing drags come with the
  editor's needs.

## Window settings in the configuration

- **The configuration holds the main window's start-up settings** ([Configuration](../core/configuration.md)): a
  window section with the same settings struct `Window::create` takes (title, mode, windowed size, resizable), next to
  the vsync and frame-cap settings. The project sets the defaults; the player settings can override them.
- **At runtime the `Window` handle is the one control,** and what the window reports is the truth. The player can
  change the window without the game (Esc on the web, macOS's green button, dragging the edges), so the configuration
  isn't kept in sync. An options menu applies a change through the handle and writes it to the configuration for the
  next start. Why: one source of truth.
- **Nothing is saved automatically** when the player drags or resizes the window; a game that wants it saves the size
  itself. A confirm-or-revert dialog for display changes is the game's options menu, not the engine.
- **How many pixels a high-DPI window renders** is the renderer's render scale (3D at a fraction and upscaled, UI and
  text at full sharpness; [Later](../later.md)); it matters in borderless fullscreen too.

## Handed to other topics

- **HDR output** is a door of the colour pipeline ([Renderer architecture](../rendering/architecture.md)); it works in
  a borderless window, so it doesn't need an exclusive mode.
- **Mobile specifics** (safe areas, orientation) come with mobile ([Later](../later.md)).
- **Native popups, owner and modal windows** come with the editor's design (egui draws its panels inside a window
  first).
- **Monitors:** a query for the monitors and the window's current one is enough for fullscreen. It asks the
  platform each time, since monitors come and go, and a monitor's refresh rate and name are optional, since platforms
  don't always report them (snapping and the frame cap's divisor rule handle an unknown rate).
