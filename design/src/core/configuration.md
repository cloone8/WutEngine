# Configuration

How engine and game settings are defined, layered, read at start-up, set from code, the console and the command line,
and locked by the game.

## The files

| File | Where | Written by |
|---|---|---|
| `config.toml`, `config.<platform>.toml`, `config.<build kind>.toml` | The project, committed | The developer (editor or by hand) |
| Resolved config | Shipped with the game (next to the executable; on the web fetched by the loader library) | The build: the project files merged for one platform and build kind |
| Player settings | Per-user storage (a file on desktop, `localStorage` on the web) | The game's options menu, or the player by hand |
| Editor preferences, per-user project state | The user's folder; an ignored folder in the project | The editor |

The command line and the console are layers too, without files.

## Typed sections, one read path

- **A setting lives in a section:** a Rust struct with the reflection derive ([Reflection](../world/reflection.md)),
  registered explicitly in the game's entry point, as components are
  (`settings::register::<GraphicsSettings>("graphics")`).
- **Code reads a section by type and always gets the effective value,** with every layer applied
  (`settings::get::<GraphicsSettings>().vsync`). There is no second "raw" or "base" read. Why: a base read that skips
  overrides is a known trap.
- **String paths** (`graphics.vsync`) exist for the console, command line and editor, derived from the schema. An
  unknown path is an error that lists close matches, never a silent default. `get` on an unregistered section is an
  error too.
- **Field markers in the schema:** `player` for fields a game's options menu may change; `restart` for fields that
  only apply at the next start (GPU backend, thread count). Changing a `restart` field in the editor or an options menu
  shows "applies after restart".
- **Who adds sections:** the engine and project Rust code, by explicit registration.

## Layers

- **The effective value is built in a fixed order, later wins:** engine defaults (the type's `Default`), project
  settings, platform override, build-kind override, player settings, command line, console and editor (this session
  only). **Every layer above the project is a sparse patch** storing only the fields it changes.
- **Each value remembers which layer set it,** so the console and editor show "set by: player settings".
  Why: no numeric priorities to reason about; the order is fixed.
- **Override files per platform and per build kind:** `config.<platform>.toml` (`windows`, `macos`, `linux`, `web`;
  later `android`, `ios`) and `config.<build kind>.toml` (`development`, `release`), platform applied before build
  kind. No files for combinations until a use case appears.
- **One main file** with a section per area. Why: settings change rarely, and git merges edits to different lines of a
  pretty-printed file by itself.
- Kept possible: quality presets ("Low", "High") as one more patch layer between build kind and player settings.

## Format: TOML for configuration, JSON for data

Configuration files (main, overrides, player settings) are TOML; scenes, materials, asset settings and other data the
editor owns stay JSON. Both store field names, so reflection's rules (fields by name, unknown fields kept) work with
either. Why: configuration is written by hand more than any other project file, and comments matter there ("locked
because of the PSX look"); every project already contains TOML (`Cargo.toml`). Costs: TOML has no `null`, so "off"
values are enums (`FrameCap::Off`) rather than `Option`; the editor needs a format-preserving write path that keeps
comments and layout.

## Start-up

Settings such as window mode, size, vsync, frames in flight and the GPU backend are needed before the window and
surface exist, and log levels before logging starts. So reading the files is separated from typing the values:

1. **Read the raw layers:** every layer is parsed into an untyped TOML table: the project config (play mode: the
   project's files; desktop builds: the shipped file next to the executable; web: handed in by the JavaScript loader
   library, which fetches it with the `.wasm`), the player settings (a file on desktop, `localStorage` on the web, both
   synchronous) and the command line. Needs only the TOML parser; problems are buffered as messages.
2. **The engine registers its early sections** (logging, window, graphics, threads); registering deserializes the
   merged layers into the struct.
3. **Logging starts** from its section; the buffered messages are written out.
4. **The game's registrations** come from its entry point, before or during start-up.
5. **After all registration,** sections or fields nobody registered are reported once, and kept, not dropped.
6. **The window and GPU** are created from their typed sections.

Consequences:

- **The game's identity (name, organisation) can't be a configuration value.** It comes from the build. Why: the
  player settings path depends on it, which would be circular.
- **Before the first frame, sets apply immediately; from the first frame on, at frame start** (phase 0,
  [Time and the frame](frame.md)), so a value never changes in the middle of a phase. Readers read the section when
  they need it, or ask whether it changed this frame. **No change callbacks.** Why: callbacks fire at unpredictable
  times.
- **The shipped game gets its settings resolved by the build:** project settings plus that platform's and build kind's
  overrides, written as one file shipped with the game; the player applies only its own layers on top. Not in a pack
  (fetched asynchronously on the web) and not compiled into the binary. Why: configuration is needed synchronously at
  start-up, and compiling it in would close the door on prebuilt players for script-only projects.

## The web's command line

- **The page passes it in:** the command-line layer on the web is a list of arguments the page hands to the loader
  library's start function ([Building and shipping](../editor/shipping.md)), in the desktop command line's syntax. From
  there it is the command-line layer as on desktop. What a page passes is decided by its web template.
- **The engine's own pages never read the URL,** in any build. Why: anyone sharing a link would set values for everyone
  who opens it, and pages carry query parameters the engine doesn't own. A developer who wants it writes it in their
  own page.

## The console

- **Console variables are settings:** the in-game development console, opened with the key below Escape
  ([Profiling and debugging tools](../editor/debug_tools.md)), sets any setting by path (`set graphics.vsync off`).
  Console changes last for the session unless explicitly saved.
- **Development-only variables** are sections marked development-only: registered only in play-mode and development
  players, compiled out of release players.

## Scripts

- **Sections registered from Rust reach TypeScript typed,** through the generated `.d.ts` (`config.graphics.vsync`).
- **Script-only values are read untyped by path** (`config.get("game.difficulty")`), returning a plain JavaScript value
  or `undefined`. Player settings, overrides, the console and locks apply to them too. A console `set` on an untyped
  path takes the type of the TOML literal. Known cost: a typo gives `undefined`; typed script-defined sections replace
  this later.
- **Script-only values live under one top-level table, `[game]`,** so the "nobody registered this" report skips them
  while still catching typos elsewhere. Packages nest their own tables under it (`[game.<name>]`). Why a fixed name
  rather than the game's: the game's name changes during development, and templates and packages need a fixed path.
- **Scripts can set `[game]` values** (`config.set("game.showHints", true)`); they land in the player settings layer,
  so a script-only game's own options are player settings, not saves.

## Player settings

- **Fields marked `player` are what a game's options menu may change;** the game decides which of them it shows.
  Saved as a sparse patch in per-user storage ([Save games and storage](../world/save_storage.md)), read synchronously
  at start-up.
- **The file is a trust boundary:** a value that doesn't parse or is out of its range gives a warning and falls back
  to the layer below for that field; unknown fields are kept, so a newer game version's settings survive an older one.
- **A `set` from game code on a `player` field lands in this layer, which the engine writes at the end of that frame,**
  at most once per frame (as background work on desktop, to `localStorage` on the web). There is no save call. Why: the
  quitting hooks don't run after a crash, on mobile or on the web, so a forgotten save call would lose settings. An
  options menu's Apply and Cancel are the game's own UI state: it calls `set` only on Apply. Rebinding and window
  settings follow the same rule; console sets stay in their session layer.
- **Frame latency** becomes a `player` setting in the display or graphics section.

## Editor preferences

The editor uses the same mechanism (it is a WutEngine program): per-user editor preferences, and per-user per-project
state (last opened scene, window layout) in an ignored folder of the project.

## Locking

Locks exist for a game built around a value the engine lets players change (a fixed low internal resolution, a fixed
step tuned for physics), and for values that exist in release builds but players shouldn't touch (time scale,
gravity). Server-enforced values in multiplayer are their networked form later.

- **Locks are declared in code only,** in the entry point with the registrations, so they hold from the first moment.
  Why: the resolved config ships as a plain file next to the game, so a lock in it could be removed by anyone;
  unlocking now takes modifying the binary.
- **Locks are per field, never per section.**
- **A lock carries its value** and wins over every layer, the project files included. Why: otherwise the value would
  still come from the editable shipped file.
- **The lock names its field with a typed field handle** from the reflection derive, one constant per field:
  `config::lock(GraphicsSettings::VSYNC, VsyncMode::On)`; a wrong field or value type is a compile error. The same
  handles serve `config::set` and "did this field change this frame". Acceptable fallback if the derive makes handles
  hard: a string path checked at start-up.
- **Player settings, the command line and the console can't change a locked value;** an attempt logs that the game
  locked it. In play mode and development builds the console may still force it, with a warning; release builds never.
- **The editor shows locked fields as read-only,** "locked in code", learned from the same registration run that
  generates the `.d.ts` files.
- **Projects without Rust code can't lock settings.** Why: a script API for locks would sit in shipped JavaScript, as
  easy to edit as the config file.
- **Not security against determined players:** it raises the bar from editing a text file to modifying the program.
