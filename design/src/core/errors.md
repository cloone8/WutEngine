# Error handling

How the engine treats bugs, misuse and failures from the outside world: what is asserted, logged, returned or fatal,
and what the player and the editor see.

## Direction

- **Scripts never crash the engine.** Invalid data from a script can produce an invalid result, never a panic:
  uploading a mesh with invalid triangles logs errors, and the mesh isn't rendered. Since a panic always ends the game
  (below), no script input may reach an assert.
- **Engine-internal code asserts liberally** to prevent invalid states.
- **Log rather than crash** wherever an error can be logged.
- **The API isn't shaped around engine-internal programming mistakes.** A simpler API that logs errors and assumes
  correct use, or enforces it with the type system, is preferred over one that makes callers handle such errors.
- **A gradient by level:** the lower-level the code, the more it asserts and returns errors; the higher-level, the less
  it asserts and the more it logs.

## Who handles what

- **Errors the caller can act on are returned:** a corrupt save file, a failed download, an asset that failed to load
  come back as `Result` in Rust and as a rejected promise or thrown error in scripts.
- **Misuse of the API is logged, not returned.** Rust and script API functions validate their input, log an error and
  degrade (a no-op, a placeholder, an unrendered mesh). Validating Rust functions are `#[track_caller]`, so the log
  names the caller's line; script errors carry the `.ts` stack trace. Repeated errors are logged once per call site
  with a count, and the editor console collapses repeats.
- **Misuse with no meaningful result** (reading a destroyed entity's position): scripts throw, caught at the callback
  like any script error; Rust logs and returns a neutral value (zero, identity, empty).
  Why: a panic is against the API rule, and `Result` everywhere against the simple API.
- **The game's own Rust code follows the same rules** when calling the API. A panic in the game's Rust code is the
  game's decision and is respected: the engine doesn't catch it (no `catch_unwind`).
- **`assert!` by default, `debug_assert!` only where the check would cost too much performance.** Accepted: an
  internal assert that fires in a shipped game ends it. Why: most checks are cheap, help the compiler optimize, and are
  wanted in shipped builds.

## Missing assets: whose mistake decides

- **An asset a scene references, missing from the pack at runtime, is an engine mistake: a hard error.** The build
  checked it, so this is a damaged install.
- **An asset the game loads dynamically** (by path or ID from code) that isn't there is the user's mistake: the load
  returns an error to the caller.
- **In play mode,** a scene can still reference an asset that was deleted, renamed or not yet committed: logged as an
  error naming the scene, the object and the asset, with a visible placeholder. **The build fails** with the same
  message, so a dangling reference can't ship.
- **A damaged install** (a quarantined file, an interrupted download) is shown to the player as "game files are
  damaged, reinstall or verify the files", not as a crash.
- **A pack download failing on the web:** a few retries, then a player-facing error ("couldn't load game data, check
  your connection and reload").

## Script errors

- **A script error skips the rest of that callback** (JavaScript's own behaviour); the error is logged with its stack
  trace and a count; the object keeps being called next frame; the frame goes on. Unhandled promise rejections are
  logged the same way. Why: disabling the failing component would be surprising, and the error is already visible.
- **The editor has a "pause on error" toggle,** off by default.
- **Runaway scripts:** boa's loop-iteration limit is on in every build, set high so a stuck `while` loop becomes an
  ordinary script error; the count is per function call, so nested loops in one function add up. The recursion limit
  stays, configurable. Why: boa counts iterations anyway, so the limit costs nothing.
- **No time watchdog** (boa can't be interrupted); in play mode the editor offers to stop a game that stopped
  responding, detected by a heartbeat over the editor link.
- **"Never" where reasonable:** scripts running out of memory or a bug in boa itself may still end the game.

## Panics and fatal errors

- **A panic is a bug:** the program is wrong and its state may be corrupt, so it aborts where it is, after the panic
  hook has logged; no flush steps, no game code. **`fatal_error(message)` is the outside world:** the program is right
  but can't go on (no usable GPU, device lost, out of graphics memory, a damaged install, a failed web download), and
  its state is intact. It logs, shows a message worded for the player and exits cleanly through the flush path
  ([Lifecycle and memory](lifecycle.md)); games can call it too.
  Why both: they differ in what the player is told (a crash to report vs. an action such as updating drivers), in data
  safety (a panic cuts off a save being written; `fatal_error` finishes pending writes), in crash reports and in what
  the editor shows.
- **`panic = "abort"` in every profile,** development included (Cargo ignores it for tests). Why: a panic then behaves
  the same in play mode, in release and on the web (where it always aborts), and a panicking worker thread can't die
  silently.
- **A panic hook** (it runs before the abort on every target) writes the message, location, a backtrace (native only)
  and the last log lines to the log, and flushes it.
- **What the player sees on a crash:** on desktop a native message box ("&lt;Game&gt; crashed. Details were written to
  &lt;log path&gt;."), then exit. On the web the page stops the frame loop and replaces the canvas with a message and
  "reload the page". The release player always writes a log file and keeps the previous run's
  ([Save games and storage](../world/save_storage.md)).
- **Exits that skip the panic hook** (out of memory, stack overflow, native crashes) can only be recorded by a monitor
  process; crash dumps and uploading come later.

## GPU errors

- **Device loss is reported and the game exits** through `fatal_error` ("the graphics device stopped responding"). In
  the editor, unsaved work is lost until crash recovery exists ([Editor](../editor/README.md)).
- **A wgpu validation error is an engine bug,** since the engine validates first: treated as a failed assert.
- **GPU out of memory when creating a buffer or texture is fatal for now,** through `fatal_error` ("not enough
  graphics memory"). Smarter degradation (evicting unused assets, dropping mip levels, retrying) comes with streaming
  and budgets. Why fatal for now: until then a failed resource needn't be marked and avoided, since the game ends.

## The editor and play mode

- **Play-mode errors stream over the editor link,** rate-limited and clickable to the source line.
- **On a play-mode crash** the panic hook sends the message over the link before the abort; the editor shows it with
  the last log lines. An exit that skipped the hook shows as "play mode ended unexpectedly (exit code N)" with the log
  tail.
- **Edit-mode script errors in the editor** follow the script-error rules. A panic in a Rust editor extension ends the
  editor (accepted).
- Kept possible: a strict mode that makes logged errors fatal, for automated runs.
