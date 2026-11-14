# Profiling and debugging tools

The tools for finding out what a game is doing: profiling, stats, debug drawing, logging, pausing and stepping, the
in-game console and runtime inspection. The basics come early, because they help build everything else.

Release players contain none of these tools; play mode and development builds do ([Building and
shipping](shipping.md)). Related tools live with their topics: the render graph's tooling ([Renderer
architecture](../rendering/architecture.md)), render debug views ([Renderer](../rendering/README.md)) and script
debugging ([Scripting](../scripting/README.md)).

## Decisions

1. **CPU profiling:** engine code is instrumented with scopes, with puffin as the CPU profiler. **Every script callback
   gets a scope automatically** (class and method name), so script time shows up without users doing anything;
   scripts can add their own scopes. Viewed in two places: the in-game overlay (everywhere, the web included) and the
   editor, which receives the play-mode player's frames through the editor link. Why: one protocol instead of a
   separate profiler server, which couldn't run on the web anyway.
2. **GPU timing:** timestamp queries per render-graph pass where supported, shown as a GPU track in the same profiler.
3. **Stats:** frame time (CPU and GPU), draw calls, triangles, entity and script counts, script time, GPU memory
   (tracked by the engine's own allocations), assets loading; in the overlay and the editor.
4. **Debug drawing:** an immediate-mode API from Rust and scripts: lines, arrows, boxes, spheres and text in the
   world; for one frame or a duration; depth-tested or on top. Drawn by an engine pass at an injection point, which
   dogfoods the render graph. A no-op in release builds. Toggles for built-in views (colliders, bounds, lights).
5. **Logging:** `log` in Rust, `console.log` and friends in scripts, both with source location, levels and categories;
   the editor console filters and jumps to the source line. Development builds show errors in the overlay. API misuse
   is logged at the caller's line, repeated errors once per call site with a count; the editor console collapses
   repeats; play-mode errors reach the editor rate-limited; the editor has a "pause on error" toggle, off by default
   ([Error handling](../core/errors.md)).
6. **Pause, step, time scale:** pause the game, advance one frame or one fixed step, change the time scale; from the
   editor and the overlay. This pause freezes all game logic and all sound ([Time and the frame](../core/frame.md)).
7. **An in-game console in development builds:** built-in commands (stats, debug views, time scale, load scene) plus
   commands and variables registered explicitly from Rust or scripts. Variables are configuration settings
   ([Configuration](../core/configuration.md)). **Toggle key: the physical key below Escape** (left of 1), whatever
   character it types, rebindable as a setting. Why: backtick on US layouts is `^`, `²` or `§` on many others. The
   console lives in the egui development overlay, so it doesn't wait for in-game UI; the overlay and console take raw
   input first while open, so typing doesn't fire game actions.
8. **Runtime inspection:** the editor shows the running game's hierarchy and inspector through the editor link, with
   live, editable values. Edits to the running game's state end with play mode; edits to the edited scenes are live
   edits and are kept ([Editor](README.md)).

## Order

First wave: logging, the in-game console, CPU profiling, stats, debug drawing, pause, step and time scale. Second wave:
GPU timing, runtime inspection, memory details.
